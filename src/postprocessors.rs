//! Postprocessors for converting Obsidian exports to Zola format.

use obsidian_export::{Context, MarkdownEvents, PostprocessorResult};
use obsidian_export::pulldown_cmark::{Event, Tag, CowStr};
use std::path::{Path, PathBuf};

/// Creates a postprocessor that converts markdown links to Zola's internal link format.
/// 
/// This function returns a postprocessor closure that has access to the source vault directory
/// for proper path resolution.
/// 
/// # Arguments
/// 
/// * `source_dir` - The path to the source vault directory
/// 
/// # Returns
/// 
/// A postprocessor function that can be used with obsidian-export
pub fn create_zola_link_postprocessor(source_dir: PathBuf) -> impl Fn(&mut Context, &mut MarkdownEvents<'_>) -> PostprocessorResult {
    move |context: &mut Context, events: &mut MarkdownEvents<'_>| {
        for event in events.iter_mut() {
            match event {
                Event::Start(Tag::Link { link_type, dest_url, title, id }) => {
                    let new_dest = convert_to_zola_link_with_context(dest_url.as_ref(), context, &source_dir);
                    *event = Event::Start(Tag::Link { 
                        link_type: *link_type, 
                        dest_url: CowStr::Boxed(new_dest.into_boxed_str()), 
                        title: title.clone(), 
                        id: id.clone()
                    });
                },
                Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
                    let new_dest = convert_to_zola_image_with_context(dest_url.as_ref(), context, &source_dir);
                    *event = Event::Start(Tag::Image { 
                        link_type: *link_type, 
                        dest_url: CowStr::Boxed(new_dest.into_boxed_str()), 
                        title: title.clone(), 
                        id: id.clone()
                    });
                },
                _ => {}
            }
        }

        PostprocessorResult::Continue
    }
}

/// Converts a markdown link URL to Zola's internal link format with proper path resolution.
/// 
/// # Arguments
/// 
/// * `url` - The original URL from the markdown link
/// * `context` - The context containing information about the current file
/// * `source_dir` - The source vault directory for proper path resolution
/// 
/// # Returns
/// 
/// A string with the converted URL. Internal .md links are properly resolved relative
/// to the current file's location and converted to `@/` format.
pub fn convert_to_zola_link_with_context(url: &str, context: &Context, source_dir: &Path) -> String {
    // Don't process external URLs (http/https/ftp/mailto etc.)
    if url.contains("://") || url.starts_with("mailto:") {
        return url.to_string();
    }
    
    // Check if this is a markdown file (with or without fragment)
    let (path_part, fragment) = if let Some(fragment_pos) = url.find('#') {
        (&url[..fragment_pos], Some(&url[fragment_pos..]))
    } else {
        (url, None)
    };
    
    let is_markdown = path_part.ends_with(".md");
    
    if is_markdown {
        // Get the current file's path
        let current_file_path = context.current_file();
        
        // Strip the source directory from the current file path to get vault-relative path
        let relative_current_file = current_file_path
            .strip_prefix(source_dir)
            .unwrap_or(current_file_path);
        
        let current_dir = relative_current_file.parent().unwrap_or_else(|| Path::new(""));
        
        // Resolve the relative path from the current file's directory
        let resolved_path = resolve_relative_path(current_dir, path_part);
        
        let fragment_suffix = fragment.unwrap_or("");
        format!("@/{}{}", resolved_path, fragment_suffix)
    } else {
        url.to_string()
    }
}

/// Converts an image URL to Zola's internal link format with proper path resolution.
/// 
/// # Arguments
/// 
/// * `url` - The original URL from the image
/// * `context` - The context containing information about the current file
/// * `source_dir` - The source vault directory for proper path resolution
/// 
/// # Returns
/// 
/// A string with the converted URL. Internal images are properly resolved relative
/// to the current file's location and converted to proper path format.
pub fn convert_to_zola_image_with_context(url: &str, context: &Context, source_dir: &Path) -> String {
    // Don't process external URLs (http/https/ftp/mailto etc.)
    if url.contains("://") || url.starts_with("mailto:") || url.starts_with("data:") {
        return url.to_string();
    }
    
    // Get the current file's path
    let current_file_path = context.current_file();
    
    // Strip the source directory from the current file path to get vault-relative path
    let relative_current_file = current_file_path
        .strip_prefix(source_dir)
        .unwrap_or(current_file_path);
    
    let current_dir = relative_current_file.parent().unwrap_or_else(|| Path::new(""));
    
    // Resolve the relative path from the current file's directory
    let resolved_path = resolve_relative_path(current_dir, url);
    
    // Special handling for static/ paths - convert to root-relative
    if resolved_path.starts_with("static/") {
        format!("/{}", &resolved_path[7..]) // Remove "static/" and add leading "/"
    } else {
        // For other images, use regular path format (not @/ which is for content links)
        resolved_path
    }
}

/// Resolves a relative path from a given current directory to an absolute path 
/// relative to the content root.
/// 
/// # Arguments
/// 
/// * `current_dir` - The current file's directory relative to content root
/// * `relative_path` - The relative path to resolve
/// 
/// # Returns
/// 
/// An absolute path relative to the content root (without leading slash)
fn resolve_relative_path(current_dir: &std::path::Path, relative_path: &str) -> String {
    // Join the current directory with the relative path
    let joined = current_dir.join(relative_path);
    
    // Normalize the path by resolving . and .. components
    let mut components = Vec::new();
    
    for component in joined.components() {
        match component {
            std::path::Component::Normal(name) => {
                components.push(name.to_string_lossy().to_string());
            }
            std::path::Component::ParentDir => {
                // Remove the last component if possible (go up one directory)
                components.pop();
            }
            std::path::Component::CurDir => {
                // Ignore current directory references
            }
            _ => {
                // Keep other components as-is
                components.push(component.as_os_str().to_string_lossy().to_string());
            }
        }
    }
    
    components.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resolve_relative_path() {
        use std::path::Path;
        
        // Test from root directory
        assert_eq!(resolve_relative_path(Path::new(""), "example.md"), "example.md");
        assert_eq!(resolve_relative_path(Path::new(""), "folder/note.md"), "folder/note.md");
        
        // Test from subdirectory
        assert_eq!(resolve_relative_path(Path::new("folder"), "../index.md"), "index.md");
        assert_eq!(resolve_relative_path(Path::new("folder"), "note.md"), "folder/note.md");
        assert_eq!(resolve_relative_path(Path::new("deep/nested"), "../../root.md"), "root.md");
        assert_eq!(resolve_relative_path(Path::new("deep/nested"), "../other.md"), "deep/other.md");
        
        // Test complex paths
        assert_eq!(resolve_relative_path(Path::new("a/b/c"), "../../d/file.md"), "a/d/file.md");
        assert_eq!(resolve_relative_path(Path::new("folder"), "./current.md"), "folder/current.md");
    }
} 