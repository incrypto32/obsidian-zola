//! Command-line interface for obsidian-zola.
//! 
//! This tool exports Obsidian notes to Zola static site generator format,
//! converting wikilinks to Zola's internal link format.

use clap::{Parser, Subcommand};
use eyre::{Result, WrapErr};
use obsidian_export::{Exporter, FrontmatterStrategy};
use obsidian_zola::{postprocessors::create_zola_link_postprocessor, utils::validate_directory};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "obsidian-zola")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export Obsidian vault to Zola format
    Export {
        /// Path to the Obsidian vault to export
        #[arg(short, long)]
        source: PathBuf,
        
        /// Path to the Zola content directory to export to
        #[arg(short, long)]
        destination: PathBuf,
        
        /// Skip processing frontmatter
        #[arg(long)]
        skip_frontmatter: bool,
        
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> Result<()> {
    // Install color-eyre for better error handling
    color_eyre::install()?;
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Export {
            source,
            destination,
            skip_frontmatter,
            verbose,
        } => {
            export_vault(source, destination, skip_frontmatter, verbose)?;
        }
    }
    
    Ok(())
}

fn export_vault(
    source: PathBuf,
    destination: PathBuf, 
    skip_frontmatter: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("🚀 Starting Obsidian to Zola export...");
        println!("📂 Source: {}", source.display());
        println!("📁 Destination: {}", destination.display());
    }
    
    // Validate input paths
    validate_directory(&source, "Source vault")
        .wrap_err("Failed to validate source vault")?;
    
    // Create destination directory if it doesn't exist
    if !destination.exists() {
        if verbose {
            println!("📁 Creating destination directory...");
        }
        std::fs::create_dir_all(&destination)
            .wrap_err("Failed to create destination directory")?;
    } else {
        validate_directory(&destination, "Destination directory")
            .wrap_err("Failed to validate destination directory")?;
    }
    
    // Set up the exporter
    let mut exporter = Exporter::new(source.clone(), destination.clone());
    
    // Configure frontmatter processing
    if skip_frontmatter {
        exporter.frontmatter_strategy(FrontmatterStrategy::Never);
        if verbose {
            println!("⏭️  Skipping frontmatter processing");
        }
    } else {
        exporter.frontmatter_strategy(FrontmatterStrategy::Always);
        if verbose {
            println!("📝 Processing frontmatter");
        }
    }
    
    // Add the Zola link postprocessor
    let zola_postprocessor = create_zola_link_postprocessor(source.clone());
    exporter.add_postprocessor(&zola_postprocessor);
    if verbose {
        println!("🔗 Added Zola link postprocessor");
    }
    
    // Run the export
    if verbose {
        println!("⚡ Running export...");
    }
    
    exporter.run()
        .wrap_err("Export failed")?;
    
    if verbose {
        println!("✅ Export completed successfully!");
        println!("🌐 Your Obsidian notes have been converted to Zola format");
        println!("📋 Internal markdown links are now using Zola's @/ format");
    } else {
        println!("Export completed successfully!");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_export_vault_creates_destination() {
        let temp_source = TempDir::new().unwrap();
        let temp_dest = TempDir::new().unwrap();
        let dest_path = temp_dest.path().join("content");
        
        // Create a simple markdown file in source
        let source_file = temp_source.path().join("test.md");
        fs::write(&source_file, "# Test\n\nThis is a test note.").unwrap();
        
        let result = export_vault(
            temp_source.path().to_path_buf(),
            dest_path.clone(),
            false,
            false,
        );
        
        assert!(result.is_ok());
        assert!(dest_path.exists());
    }
    
    #[test]
    fn test_export_vault_invalid_source() {
        let temp_dest = TempDir::new().unwrap();
        let invalid_source = PathBuf::from("/nonexistent/path");
        
        let result = export_vault(
            invalid_source,
            temp_dest.path().to_path_buf(),
            false,
            false,
        );
        
        assert!(result.is_err());
    }
}
