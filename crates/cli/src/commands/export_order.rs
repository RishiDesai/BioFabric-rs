//! `biofabric export-order` — export node or link ordering.

use crate::args::ExportOrderArgs;

pub fn run(_args: ExportOrderArgs) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement export-order command
    //
    // 1. Load session or layout JSON
    //    let session = FabricFactory::load_session(&args.input)?;
    //    let layout = session.layout.ok_or("No layout in session")?;
    //
    // 2. Export
    //    let writer: Box<dyn Write> = if let Some(path) = &args.output {
    //        Box::new(std::fs::File::create(path)?)
    //    } else {
    //        Box::new(std::io::stdout())
    //    };
    //
    //    match args.what {
    //        OrderExportType::Nodes => io::order::write_node_order(&mut writer, &layout)?,
    //        OrderExportType::Links => io::order::write_link_order(&mut writer, &layout)?,
    //    }
    //
    todo!("Implement export-order command")
}
