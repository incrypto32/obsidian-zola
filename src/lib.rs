//! # Obsidian-Zola
//! 
//! A library and CLI tool for exporting Obsidian notes to Zola static site generator format.
//! 
//! This crate provides postprocessors that convert Obsidian's `[[wikilinks]]` into Zola's
//! `@/path/to/file.md` internal link format, making it easy to migrate your Obsidian vault
//! to a Zola-powered website.
//! 
//! ## Features
//! 
//! - Convert `[[wikilinks]]` to Zola's `@/` internal links
//! - Preserve external URLs and non-markdown links
//! - Handle links with sections (`[[Note#Section]]`)
//! - Support for custom link text (`[[Note|Custom Text]]`)
//! - Comprehensive error handling and logging
//! 
//! ## Usage as Library
//! 
//! ```no_run
//! use obsidian_zola::postprocessors::create_zola_link_postprocessor;
//! use obsidian_export::Exporter;
//! use std::path::PathBuf;
//! 
//! let vault_path = PathBuf::from("path/to/vault");
//! let mut exporter = Exporter::new(
//!     vault_path.clone(),
//!     PathBuf::from("path/to/zola/content")
//! );
//! 
//! let zola_postprocessor = create_zola_link_postprocessor(vault_path);
//! exporter.add_postprocessor(&zola_postprocessor);
//! exporter.run().unwrap();
//! ```

pub mod postprocessors;
pub mod utils;

pub use postprocessors::*;

// Re-export commonly used types from obsidian-export for convenience
pub use obsidian_export::{
    Context, Exporter, FrontmatterStrategy, MarkdownEvents, PostprocessorResult
}; 