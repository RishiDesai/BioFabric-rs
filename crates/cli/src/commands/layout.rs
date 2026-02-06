//! `biofabric layout` — compute a layout for a network file.

use crate::args::LayoutArgs;

pub fn run(_args: LayoutArgs, _quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement layout command
    //
    // 1. Detect format and load network
    //    let network = FabricFactory::load_network(&args.input)?;
    //
    // 2. Optionally load attributes (for cluster/set/control-top)
    //    if let Some(attr_path) = &args.attributes {
    //        let attrs = attribute::parse_file(attr_path)?;
    //        // Apply attributes to network nodes
    //    }
    //
    // 3. Optionally load custom node order
    //    if let Some(order_path) = &args.node_order {
    //        let order = io::order::parse_node_order_file(order_path)?;
    //        // Use this order directly instead of computing one
    //    }
    //
    // 4. Generate shadows if requested
    //    let show_shadows = args.shadows && !args.no_shadows;
    //    if show_shadows { network.generate_shadows(); }
    //
    // 5. Select and configure layout algorithm
    //    match args.algorithm {
    //        LayoutAlgorithm::Default => DefaultNodeLayout::new(),
    //        LayoutAlgorithm::Similarity => NodeSimilarityLayout::new(),
    //        LayoutAlgorithm::Hierarchy => HierDAGLayout::new(),
    //        LayoutAlgorithm::Cluster => {
    //            let attr = args.cluster_attribute.expect("--cluster-attribute required");
    //            let assignments = attrs.group_by(&attr);
    //            NodeClusterLayout::new(assignments)
    //        },
    //        LayoutAlgorithm::ControlTop => {
    //            let attr = args.control_attribute.expect("--control-attribute required");
    //            let nodes = attrs.nodes_with_value(&attr, args.control_value.as_deref().unwrap_or(""));
    //            ControlTopLayout::new(nodes)
    //        },
    //        LayoutAlgorithm::Set => SetLayout::new(),
    //        LayoutAlgorithm::WorldBank => WorldBankLayout::new(),
    //    }
    //
    // 6. Optionally apply custom link group order
    //    if let Some(order) = &args.link_group_order {
    //        // Use this order for the edge layout's link group ordering
    //    }
    //
    // 7. Build LayoutParams
    //    let params = LayoutParams {
    //        start_node: args.start_node.map(NodeId::new),
    //        include_shadows: show_shadows,
    //        layout_mode: match args.link_group_mode { ... },
    //        ...
    //    };
    //
    // 8. Compute layout (node phase + edge phase)
    //    let monitor = CliProgressMonitor::new(quiet);
    //    let node_order = node_layout.layout_nodes(&network, &params, &monitor)?;
    //    let mut build_data = LayoutBuildData::new(network, node_order, show_shadows, params.layout_mode);
    //    let layout = edge_layout.layout_edges(&mut build_data, &params, &monitor)?;
    //
    // 9. Write output
    //    match output extension:
    //      .json → write layout as JSON
    //      .bif/.xml → save full session
    //      stdout → print layout JSON
    //
    todo!("Implement layout command")
}
