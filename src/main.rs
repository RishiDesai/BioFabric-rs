//! BioFabric CLI
//!
//! Command-line interface for BioFabric network visualization.

use biofabric::{io, Network};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: biofabric <input.sif> [--output <output.json>]");
        eprintln!();
        eprintln!("Supported formats:");
        eprintln!("  .sif  - Simple Interaction Format");
        eprintln!("  .gw   - LEDA Graph Format");
        return ExitCode::FAILURE;
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = args
        .iter()
        .position(|a| a == "--output")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);

    // Determine format from extension
    let extension = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let network = match extension.to_lowercase().as_str() {
        "sif" => match io::sif::parse_file(&input_path) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error parsing SIF file: {}", e);
                return ExitCode::FAILURE;
            }
        },
        "gw" => match io::gw::parse_file(&input_path) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error parsing GW file: {}", e);
                return ExitCode::FAILURE;
            }
        },
        _ => {
            eprintln!("Unsupported file format: {}", extension);
            return ExitCode::FAILURE;
        }
    };

    println!("Loaded network: {} nodes, {} links", network.node_count(), network.link_count());

    // If output path specified, write JSON
    if let Some(out_path) = output_path {
        match network.to_json_file(&out_path) {
            Ok(_) => println!("Written to {:?}", out_path),
            Err(e) => {
                eprintln!("Error writing output: {}", e);
                return ExitCode::FAILURE;
            }
        }
    } else {
        // Print to stdout
        match serde_json::to_string_pretty(&network) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Error serializing: {}", e);
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
