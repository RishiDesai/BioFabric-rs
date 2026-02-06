//! `biofabric convert` — convert a network between file formats.

use crate::args::ConvertArgs;

pub fn run(_args: ConvertArgs, _quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement convert command
    //
    // 1. Load network
    //    let mut network = FabricFactory::load_network(&args.input)?;
    //
    // 2. Strip shadows unless --keep-shadows
    //    if !args.keep_shadows {
    //        network.links_mut().retain(|l| !l.is_shadow);
    //    }
    //
    // 3. Determine output format
    //    let out_format = match args.format {
    //        ConvertFormat::Sif => OutputFormat::Sif,
    //        ConvertFormat::Gw => OutputFormat::Gw,
    //        ConvertFormat::Json => OutputFormat::Json,
    //        ConvertFormat::Xml => OutputFormat::Xml,
    //    };
    //
    // 4. Write output
    //    if let Some(path) = &args.output {
    //        FabricFactory::write_network(&network, out_format, path)?;
    //    } else {
    //        let s = FabricFactory::write_network_string(&network, out_format)?;
    //        print!("{}", s);
    //    }
    //
    todo!("Implement convert command")
}
