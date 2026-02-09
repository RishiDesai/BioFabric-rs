//! Set membership layout algorithm.
//!
//! Designed for bipartite-style networks where one class of nodes represents
//! "sets" and the other class represents "members". For example, gene
//! ontology terms (sets) and genes (members).
//!
//! ## Semantics
//!
//! - **BelongsTo** - Members belong to sets (edges go member -> set)
//! - **Contains** - Sets contain members (edges go set -> member)
//!
//! ## Algorithm
//!
//! 1. Identify set nodes and member nodes based on relation semantics
//! 2. Order sets by cardinality (number of members), then name
//! 3. For each member, compute a Gray-code-based sort key from its
//!    set membership bitvector
//! 4. Order members by (degree DESC, gray code DESC, name ASC)
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.layouts.SetLayout`

use super::traits::{LayoutParams, LayoutResult, NodeLayout};
use crate::model::{Network, NodeId};
use crate::worker::ProgressMonitor;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Relationship semantics for set membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetSemantics {
    /// Members belong to sets (edge: member -> set).
    #[default]
    BelongsTo,
    /// Sets contain members (edge: set -> member).
    Contains,
}

/// Configuration for the set layout.
#[derive(Debug, Clone, Default)]
pub struct SetLayoutParams {
    /// How to interpret edges.
    pub semantics: SetSemantics,
    /// The relation type that indicates set membership.
    /// If `None`, all relations are treated as membership.
    pub membership_relation: Option<String>,
}

/// Set membership layout.
///
/// Orders nodes so that set nodes and their members are adjacent,
/// with sets ordered by cardinality.
#[derive(Debug, Clone, Default)]
pub struct SetLayout {
    /// Layout configuration.
    pub config: SetLayoutParams,
}

impl SetLayout {
    /// Create a new set layout with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the membership semantics.
    pub fn with_semantics(mut self, semantics: SetSemantics) -> Self {
        self.config.semantics = semantics;
        self
    }

    /// Set the membership relation type.
    pub fn with_relation(mut self, relation: impl Into<String>) -> Self {
        self.config.membership_relation = Some(relation.into());
        self
    }

    /// Extract set→members mapping from the network.
    ///
    /// Returns `(elems_per_set, sets_per_elem)` where:
    /// - `elems_per_set`: set node → set of member nodes
    /// - `sets_per_elem`: member node → set of set nodes
    fn extract_sets(
        &self,
        network: &Network,
    ) -> (
        HashMap<NodeId, BTreeSet<NodeId>>,
        HashMap<NodeId, BTreeSet<NodeId>>,
    ) {
        let mut elems_per_set: HashMap<NodeId, BTreeSet<NodeId>> = HashMap::new();
        let mut sets_per_elem: HashMap<NodeId, BTreeSet<NodeId>> = HashMap::new();

        for link in network.links() {
            if link.is_shadow {
                continue;
            }

            let (set_node, member_node) = match self.config.semantics {
                SetSemantics::BelongsTo => (&link.target, &link.source),
                SetSemantics::Contains => (&link.source, &link.target),
            };

            elems_per_set
                .entry(set_node.clone())
                .or_default()
                .insert(member_node.clone());
            sets_per_elem
                .entry(member_node.clone())
                .or_default()
                .insert(set_node.clone());
        }

        (elems_per_set, sets_per_elem)
    }

    /// Order sets by cardinality descending, then name ascending.
    ///
    /// Matches Java `SetLayout.doNodeLayout()`:
    /// - Groups sets into a TreeMap keyed by -cardinality (reverse order)
    /// - Within each group, sets are in TreeSet (name-ascending) order
    fn order_sets(elems_per_set: &HashMap<NodeId, BTreeSet<NodeId>>) -> Vec<NodeId> {
        // Group by cardinality (BTreeMap with reverse key for descending order)
        let mut by_card: BTreeMap<std::cmp::Reverse<usize>, BTreeSet<&NodeId>> = BTreeMap::new();
        for (set_node, members) in elems_per_set {
            by_card
                .entry(std::cmp::Reverse(members.len()))
                .or_default()
                .insert(set_node);
        }

        let mut result = Vec::new();
        for (_card, sets) in &by_card {
            for set_node in sets {
                result.push((*set_node).clone());
            }
        }
        result
    }

    /// Order members using Gray-code-based sort key.
    ///
    /// For each member, compute a bitvector based on which sets (from
    /// `set_list`) it belongs to. Convert bitvector to binary number,
    /// then to Gray code. Sort by (degree DESC, gray DESC, name ASC).
    ///
    /// Matches Java `SetLayout.nodeGraySetWithSourceFromMap()` and
    /// `SourcedNodeGray.compareTo()`.
    fn order_members(
        set_list: &[NodeId],
        sets_per_elem: &HashMap<NodeId, BTreeSet<NodeId>>,
        network: &Network,
    ) -> Vec<NodeId> {
        // Compute degree for each member (from non-shadow links)
        let mut degree: HashMap<NodeId, usize> = HashMap::new();
        for link in network.links() {
            if link.is_shadow {
                continue;
            }
            *degree.entry(link.source.clone()).or_insert(0) += 1;
            *degree.entry(link.target.clone()).or_insert(0) += 1;
        }

        // Compute Gray code for each member
        struct MemberKey {
            id: NodeId,
            degree: usize,
            gray: u64,
        }

        let mut members: Vec<MemberKey> = sets_per_elem
            .keys()
            .map(|member| {
                let member_sets = &sets_per_elem[member];

                // Build bitvector: bit i = 1 if member belongs to set_list[i]
                let mut binary_val: u64 = 0;
                for (i, set_node) in set_list.iter().enumerate() {
                    if member_sets.contains(set_node) {
                        binary_val |= 1 << (set_list.len() - 1 - i);
                    }
                }

                // Convert binary to Gray code: gray = n XOR (n >> 1)
                let gray = binary_val ^ (binary_val >> 1);

                MemberKey {
                    id: member.clone(),
                    degree: degree.get(member).copied().unwrap_or(0),
                    gray,
                }
            })
            .collect();

        // Sort: degree DESC, gray DESC, name ASC
        // But Java uses TreeSet with compareTo that does degree ASC, gray ASC(?), name ASC
        // then reverses the entire list.
        //
        // Actually, from the analysis:
        // Java SourcedNodeGray.compareTo: degree ASC, gray/bi comparison, name ASC
        // Then Collections.reverse() is called.
        // After reversal: degree DESC, gray DESC, name DESC within ties
        //
        // Wait -- let me re-check. The golden says:
        // BelongsTo members: L2, L1, L3
        //   L2: degree=2, gray=7
        //   L1: degree=2, gray=5
        //   L3: degree=2, gray=2
        // All have same degree=2. Sorted by gray DESC: L2(7), L1(5), L3(2). ✓
        //
        // For name tiebreaking, since we reverse: name DESC (but there are no ties
        // in this test case).
        //
        // Java pattern: TreeSet(asc) + reverse = desc for all keys
        // So: degree ASC + reverse = degree DESC
        //     gray ASC + reverse = gray DESC
        //     name ASC + reverse = name DESC
        //
        // Final: degree DESC, gray DESC, name DESC
        members.sort_by(|a, b| {
            b.degree
                .cmp(&a.degree)
                .then(b.gray.cmp(&a.gray))
                .then(b.id.cmp(&a.id))
        });

        members.into_iter().map(|m| m.id).collect()
    }
}

impl NodeLayout for SetLayout {
    fn layout_nodes(
        &self,
        network: &Network,
        _params: &LayoutParams,
        _monitor: &dyn ProgressMonitor,
    ) -> LayoutResult<Vec<NodeId>> {
        let (elems_per_set, sets_per_elem) = self.extract_sets(network);

        // Order sets by cardinality DESC, name ASC
        let set_list = Self::order_sets(&elems_per_set);

        // Order members by Gray code
        let member_list = Self::order_members(&set_list, &sets_per_elem, network);

        // Combine: sets first, then members
        let mut result = Vec::with_capacity(set_list.len() + member_list.len());
        result.extend(set_list);
        result.extend(member_list);

        // Add any lone nodes not yet placed
        let placed: HashSet<NodeId> = result.iter().cloned().collect();
        for id in network.node_ids() {
            if !placed.contains(id) {
                result.push(id.clone());
            }
        }

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "Set Membership"
    }
}
