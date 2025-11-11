use std::path::PathBuf;
use std::process::Command;

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
    let output_dir = output.unwrap_or_else(|| config.swift.output.clone());
    
    std::fs::create_dir_all(&output_dir)
        .map_err(|source| CliError::CreateDirectoryFailed {
            path: output_dir.clone(),
            source,
        })?;

    let status = Command::new("cargo")
        .args(["run", "-p", "mobiFFI_bindgen", "--", "swift", "-o"])
        .arg(&output_dir)
        .status()
        .map_err(|_| CliError::CommandFailed {
            command: "mobiFFI_bindgen swift".to_string(),
            status: None,
        })?;

    if !status.success() {
        return Err(CliError::CommandFailed {
            command: "mobiFFI_bindgen swift".to_string(),
            status: status.code(),
        });
    }

    println!("Generated Swift bindings -> {}", output_dir.display());
    Ok(())
}

fn generate_kotlin(config: &Config, output: Option<PathBuf>) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| config.kotlin.output.clone());
    
    std::fs::create_dir_all(&output_dir)
        .map_err(|source| CliError::CreateDirectoryFailed {
            path: output_dir.clone(),
            source,
        })?;

    let status = Command::new("cargo")
        .args(["run", "-p", "mobiFFI_bindgen", "--", "kotlin", "-o"])
        .arg(&output_dir)
        .status()
        .map_err(|_| CliError::CommandFailed {
            command: "mobiFFI_bindgen kotlin".to_string(),
            status: None,
        })?;

    if !status.success() {
        return Err(CliError::CommandFailed {
            command: "mobiFFI_bindgen kotlin".to_string(),
            status: status.code(),
        });
    }

    println!("Generated Kotlin bindings -> {}", output_dir.display());
    Ok(())
}

fn generate_header(config: &Config, output: Option<PathBuf>) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| PathBuf::from("include"));
    
    std::fs::create_dir_all(&output_dir)
        .map_err(|source| CliError::CreateDirectoryFailed {
            path: output_dir.clone(),
            source,
        })?;

    let lib_name = config.library_name();
    let header_path = output_dir.join(format!("{}.h", lib_name));

    let status = Command::new("cargo")
        .args(["run", "-p", "mobiFFI_bindgen", "--", "header", "-o"])
        .arg(&header_path)
        .status()
        .map_err(|_| CliError::CommandFailed {
            command: "mobiFFI_bindgen header".to_string(),
            status: None,
        })?;

    if !status.success() {
        return Err(CliError::CommandFailed {
            command: "mobiFFI_bindgen header".to_string(),
            status: status.code(),
        });
    }

    println!("Generated header -> {}", header_path.display());
    Ok(())
}
