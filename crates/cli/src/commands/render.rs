//! `biofabric render` — render a network to an image file.

use crate::args::{ImageFormatArg, RenderArgs};
use biofabric_core::export::{ExportOptions, ImageExporter, ImageFormat};
use biofabric_core::io::factory::FabricFactory;
use biofabric_core::layout::traits::{LayoutMode, LayoutParams, NetworkLayoutAlgorithm, TwoPhaseLayout};
use biofabric_core::layout::{DefaultEdgeLayout, DefaultNodeLayout};
use biofabric_core::render::gpu_data::RenderOutput;
use biofabric_core::worker::NoopMonitor;

pub fn run(args: RenderArgs, quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
    let show_shadows = args.shadows && !args.no_shadows;

    // Load input — could be a session or a network
    let ext = args.input.extension().and_then(|e| e.to_str()).unwrap_or("");
    let (_network, layout) = match ext {
        "bif" | "xml" => {
            let session = FabricFactory::load_session(&args.input)?;
            let layout = session
                .layout
                .ok_or("Session file has no saved layout")?;
            (session.network, layout)
        }
        _ => {
            let mut network = FabricFactory::load_network(&args.input)?;
            if show_shadows {
                network.generate_shadows();
            }
            let params = LayoutParams {
                include_shadows: show_shadows,
                layout_mode: LayoutMode::PerNode,
                ..Default::default()
            };
            let two_phase =
                TwoPhaseLayout::new(DefaultNodeLayout::new(), DefaultEdgeLayout::new());
            let layout = two_phase.layout(&network, &params, &NoopMonitor)?;
            (network, layout)
        }
    };

    // Auto-compute height if needed
    let height = if args.height == 0 {
        if layout.column_count == 0 {
            args.width
        } else {
            let ratio = layout.row_count as f64 / layout.column_count as f64;
            (args.width as f64 * ratio).max(1.0) as u32
        }
    } else {
        args.height
    };

    // Detect output format
    let format = if let Some(fmt) = args.format {
        match fmt {
            ImageFormatArg::Png => ImageFormat::Png,
            ImageFormatArg::Jpeg => ImageFormat::Jpeg,
            ImageFormatArg::Tiff => ImageFormat::Tiff,
        }
    } else {
        let out_ext = args.output.extension().and_then(|e| e.to_str()).unwrap_or("png");
        match out_ext {
            "jpg" | "jpeg" => ImageFormat::Jpeg,
            "tiff" | "tif" => ImageFormat::Tiff,
            _ => ImageFormat::Png,
        }
    };

    // Build a minimal RenderOutput; full render extraction (viewport
    // culling, LOD, labels) is not yet implemented in the core library.
    // We can still export a background-only image at the correct dimensions.
    let render = RenderOutput::empty();

    let export_opts = ExportOptions {
        format,
        width_px: args.width,
        height_px: height,
        dpi: args.dpi,
        background_color: args.background.clone(),
        ..Default::default()
    };

    ImageExporter::export_to_file(&render, &export_opts, &args.output, &NoopMonitor)
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    if !quiet {
        eprintln!(
            "Rendered {}x{} {} → {}",
            args.width,
            height,
            format!("{:?}", format).to_lowercase(),
            args.output.display(),
        );
    }

    Ok(())
}
