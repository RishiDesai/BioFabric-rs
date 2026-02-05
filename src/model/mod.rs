//! Core data model for BioFabric networks.
//!
//! This module contains the fundamental types:
//! - [`NodeId`] - Unique identifier for a node
//! - [`Node`] - A node in the network
//! - [`Link`] - A connection between two nodes
//! - [`Network`] - The complete network graph

mod link;
mod network;
mod node;

pub use link::Link;
pub use network::Network;
pub use node::{Node, NodeId};
