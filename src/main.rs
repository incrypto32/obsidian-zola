//! Command-line interface for obsidian-zola.
//! 
//! This tool exports Obsidian notes to Zola static site generator format,
//! converting wikilinks to Zola's internal link format.

use clap::{Parser, Subcommand};
use eyre::{Result, WrapErr};
use obsidian_export::{Exporter, FrontmatterStrategy};
use obsidian_zola::{postprocessors::create_zola_link_postprocessor, utils::validate_directory};
use std::path::PathBuf;
use std::fs;
use walkdir::WalkDir;
use glob::Pattern;

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
        
        /// Patterns for files to copy as-is without processing (can be used multiple times)
        #[arg(long = "passthrough")]
        passthrough_patterns: Vec<String>,
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
            passthrough_patterns,
        } => {
            export_vault(source, destination, skip_frontmatter, verbose, passthrough_patterns)?;
        }
    }
    
    Ok(())
}

fn export_vault(
    source: PathBuf,
    destination: PathBuf, 
    skip_frontmatter: bool,
    verbose: bool,
    passthrough_patterns: Vec<String>,
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

    // Handle passthrough files first if any patterns are specified
    if !passthrough_patterns.is_empty() {
        if verbose {
            println!("📋 Processing passthrough files...");
        }
        copy_passthrough_files(&source, &destination, &passthrough_patterns, verbose)?;
        
        // Create temporary .export-ignore file to exclude passthrough files from obsidian-export
        create_temporary_ignore_file(&source, &passthrough_patterns)?;
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
    
    // Add the Zola link postprocessor (no passthrough patterns needed since they're excluded)
    let zola_postprocessor = create_zola_link_postprocessor(source.clone());
    exporter.add_postprocessor(&zola_postprocessor);
    if verbose {
        println!("🔗 Added Zola link postprocessor");
    }
    
    // Run the export
    if verbose {
        println!("⚡ Running export...");
    }
    
    let result = exporter.run();
    
    // Clean up temporary ignore file
    if !passthrough_patterns.is_empty() {
        cleanup_temporary_ignore_file(&source);
    }
    
    result.wrap_err("Export failed")?;
    
    if verbose {
        println!("✅ Export completed successfully!");
        println!("🌐 Your Obsidian notes have been converted to Zola format");
        println!("📋 Internal markdown links are now using Zola's @/ format");
        if !passthrough_patterns.is_empty() {
            println!("📄 Passthrough files copied as-is without processing");
        }
    } else {
        println!("Export completed successfully!");
    }
    
    Ok(())
}

/// Copies files matching passthrough patterns as-is to the destination
fn copy_passthrough_files(
    source: &PathBuf, 
    destination: &PathBuf, 
    patterns: &[String], 
    verbose: bool
) -> Result<()> {
    // Compile all patterns upfront
    let compiled_patterns: Result<Vec<Pattern>, _> = patterns
        .iter()
        .map(|p| Pattern::new(p))
        .collect();
    
    let compiled_patterns = compiled_patterns
        .wrap_err("Failed to compile glob patterns")?;
    
    for entry in WalkDir::new(source) {
        let entry = entry.wrap_err("Failed to read directory entry")?;
        let path = entry.path();
        
        // Skip directories
        if path.is_dir() {
            continue;
        }
        
        // Get relative path from source
        let relative_path = path.strip_prefix(source)
            .wrap_err("Failed to get relative path")?;
        
        let path_str = relative_path.to_string_lossy();
        
        // Check if this file matches any passthrough pattern
        let should_copy = compiled_patterns.iter().any(|pattern| {
            pattern.matches(&path_str)
        });
        
        if should_copy {
            let dest_path = destination.join(relative_path);
            
            // Create parent directories if needed
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .wrap_err("Failed to create destination directory")?;
            }
            
            // Copy the file as-is
            fs::copy(path, &dest_path)
                .wrap_err("Failed to copy passthrough file")?;
            
            if verbose {
                println!("📄 Copied passthrough: {}", relative_path.display());
            }
        }
    }
    
    Ok(())
}

/// Creates a temporary .export-ignore file to exclude passthrough files from obsidian-export
fn create_temporary_ignore_file(source: &PathBuf, patterns: &[String]) -> Result<()> {
    let ignore_file = source.join(".export-ignore");
    let backup_file = source.join(".export-ignore.backup");
    
    // Back up existing .export-ignore if it exists
    if ignore_file.exists() {
        fs::copy(&ignore_file, &backup_file)
            .wrap_err("Failed to backup existing .export-ignore")?;
        
        // Read existing content
        let mut content = fs::read_to_string(&ignore_file)
            .wrap_err("Failed to read existing .export-ignore")?;
        
        // Add passthrough patterns
        content.push_str("\n# Temporary patterns for passthrough files\n");
        for pattern in patterns {
            content.push_str(pattern);
            content.push('\n');
        }
        
        fs::write(&ignore_file, content)
            .wrap_err("Failed to update ignore file")?;
    } else {
        // Create new .export-ignore with just our patterns
        let mut content = String::from("# Temporary patterns for passthrough files\n");
        for pattern in patterns {
            content.push_str(pattern);
            content.push('\n');
        }
        
        fs::write(&ignore_file, content)
            .wrap_err("Failed to create temporary ignore file")?;
    }
    
    Ok(())
}

/// Cleans up the temporary ignore file modifications
fn cleanup_temporary_ignore_file(source: &PathBuf) {
    let ignore_file = source.join(".export-ignore");
    let backup_file = source.join(".export-ignore.backup");
    
    if backup_file.exists() {
        // Restore from backup
        if let Err(e) = fs::rename(&backup_file, &ignore_file) {
            eprintln!("Warning: Failed to restore .export-ignore from backup: {}", e);
        }
    } else if ignore_file.exists() {
        // We created the file, so remove it entirely
        if let Err(e) = fs::remove_file(&ignore_file) {
            eprintln!("Warning: Failed to remove temporary .export-ignore: {}", e);
        }
    }
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
            Vec::new(),
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
            Vec::new(),
        );
        
        assert!(result.is_err());
    }
}
