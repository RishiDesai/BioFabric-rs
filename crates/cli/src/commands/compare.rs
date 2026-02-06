//! `biofabric compare` — compare the neighborhoods of two nodes.

use crate::args::CompareArgs;

pub fn run(_args: CompareArgs) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement compare command
    //
    // 1. Load network
    //    let network = FabricFactory::load_network(&args.input)?;
    //
    // 2. Compare nodes
    //    let comparison = network.compare_nodes(
    //        &NodeId::new(&args.node_a),
    //        &NodeId::new(&args.node_b),
    //    ).ok_or("Node not found")?;
    //
    // 3. Output
    //    match args.format {
    //        InfoFormat::Text => {
    //            println!("Node A: {} (degree {})", comparison.node_a, comparison.degree_a);
    //            println!("Node B: {} (degree {})", comparison.node_b, comparison.degree_b);
    //            println!("Shared neighbors ({}): {:?}", comparison.shared_neighbors.len(), ...);
    //            println!("Exclusive to A ({}): {:?}", comparison.exclusive_a.len(), ...);
    //            println!("Exclusive to B ({}): {:?}", comparison.exclusive_b.len(), ...);
    //            println!("Jaccard similarity: {:.4}", comparison.jaccard_similarity);
    //        }
    //        InfoFormat::Json => println!("{}", serde_json::to_string_pretty(&comparison)?),
    //    }
    //
    todo!("Implement compare command")
}
