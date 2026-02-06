//! `biofabric extract` — extract a subnetwork.

use crate::args::ExtractArgs;

pub fn run(_args: ExtractArgs, _quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement extract command
    //
    // 1. Load network
    //    let network = FabricFactory::load_network(&args.input)?;
    //
    // 2. Determine nodes to extract
    //    let subnetwork = if let Some(center) = &args.node {
    //        network.extract_neighborhood(&NodeId::new(center), args.hops)
    //    } else if let Some(list_path) = &args.node_list {
    //        let node_ids: HashSet<NodeId> = io::order::parse_node_order_file(list_path)?
    //            .into_iter().collect();
    //        network.extract_subnetwork(&node_ids)
    //    } else {
    //        return Err("Specify --node or --node-list".into());
    //    };
    //
    // 3. Write output
    //    let out_format = match args.format { ... };
    //    if let Some(path) = &args.output {
    //        FabricFactory::write_network(&subnetwork, out_format, path)?;
    //    } else {
    //        let s = FabricFactory::write_network_string(&subnetwork, out_format)?;
    //        print!("{}", s);
    //    }
    //
    todo!("Implement extract command")
}
