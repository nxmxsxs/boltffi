use std::path::PathBuf;

use riff_bindgen::{CHeaderGenerator, Swift, scan_crate};

use crate::config::Config;
use crate::error::{CliError, Result};

pub enum GenerateTarget {
    Swift,
    Kotlin,
    Header,
    All,
}

pub struct GenerateOptions {
    pub target: GenerateTarget,
    pub output: Option<PathBuf>,
}

pub fn run_generate(config: &Config, options: GenerateOptions) -> Result<()> {
    match options.target {
        GenerateTarget::Swift => generate_swift(config, options.output),
        GenerateTarget::Kotlin => generate_kotlin(config, options.output),
        GenerateTarget::Header => generate_header(config, options.output),
        GenerateTarget::All => {
            generate_swift(config, None)?;
            generate_kotlin(config, None)?;
            generate_header(config, None)?;
            Ok(())
        }
    }
}

fn generate_swift(config: &Config, output: Option<PathBuf>) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| PathBuf::from("dist/Sources"));
    let output_path = output_dir.join(format!("{}.swift", config.swift_module_name()));

    std::fs::create_dir_all(&output_dir).map_err(|source| CliError::CreateDirectoryFailed {
        path: output_dir.clone(),
        source,
    })?;

    let crate_dir = PathBuf::from(".");
    let crate_name = config.library_name();

    let module = scan_crate(&crate_dir, crate_name).map_err(|e| CliError::CommandFailed {
        command: format!("scan_crate: {}", e),
        status: None,
    })?;

    let swift_code = Swift::render_module(&module);

    std::fs::write(&output_path, swift_code).map_err(|source| CliError::WriteFailed {
        path: output_path.clone(),
        source,
    })?;

    println!("Generated: {}", output_path.display());
    Ok(())
}

fn generate_kotlin(_config: &Config, _output: Option<PathBuf>) -> Result<()> {
    println!("Kotlin generation not yet implemented");
    Ok(())
}

fn generate_header(config: &Config, output: Option<PathBuf>) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| PathBuf::from("dist/include"));
    let output_path = output_dir.join(format!("{}.h", config.library_name()));

    std::fs::create_dir_all(&output_dir).map_err(|source| CliError::CreateDirectoryFailed {
        path: output_dir.clone(),
        source,
    })?;

    let crate_dir = PathBuf::from(".");
    let crate_name = config.library_name();

    let module = scan_crate(&crate_dir, crate_name).map_err(|e| CliError::CommandFailed {
        command: format!("scan_crate: {}", e),
        status: None,
    })?;

    let header_code = CHeaderGenerator::generate(&module);

    std::fs::write(&output_path, header_code).map_err(|source| CliError::WriteFailed {
        path: output_path.clone(),
        source,
    })?;

    println!("Generated: {}", output_path.display());
    Ok(())
}
