//! `biofabric render` — render a network to an image file.

use crate::args::RenderArgs;

pub fn run(_args: RenderArgs, _quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement render command
    //
    // 1. Detect format and load input
    //    - If session file (.bif/.xml): load session, use saved layout
    //    - If network file: load network, compute layout with args.algorithm
    //
    // 2. Configure display options
    //    let show_shadows = args.shadows && !args.no_shadows;
    //    let display = DisplayOptions {
    //        show_shadows,
    //        show_node_labels: args.labels,
    //        show_link_labels: args.link_labels,
    //        show_annotations: args.annotations,
    //        background_color: args.background.clone(),
    //        ..DisplayOptions::for_image_export(show_shadows)
    //    };
    //
    // 3. Compute render output
    //    let palette = ColorPalette::default_palette();
    //    let render_params = RenderParams { show_shadows, ... };
    //    let render = RenderOutput::extract(&layout, &render_params, &palette);
    //
    // 4. Auto-compute height if needed
    //    let height = if args.height == 0 {
    //        (args.width as f64 * layout.row_count as f64 / layout.column_count as f64) as u32
    //    } else { args.height };
    //
    // 5. Export image
    //    let export_opts = ExportOptions {
    //        format: detect_image_format(&args.output, args.format),
    //        width_px: args.width,
    //        height_px: height,
    //        dpi: args.dpi,
    //        background_color: args.background,
    //        ..Default::default()
    //    };
    //    ImageExporter::export_to_file(&render, &export_opts, &args.output, &monitor)?;
    //
    todo!("Implement render command")
}
