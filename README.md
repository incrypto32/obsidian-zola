# obsidian-zola

A specialized tool for converting Obsidian vault exports to Zola static site generator format, with automatic wikilink conversion to Zola's internal link syntax.

## ⚠️ Personal Tool Disclaimer

**This is a personal tool built for my specific workflow and requirements.** While it's open source and you're welcome to use it, please note:

- This tool was designed for my personal Obsidian → Zola publishing workflow
- It may not work perfectly for all vault structures or use cases
- Features are implemented based on my specific needs
- Support and feature requests are limited
- Use at your own risk

## Features

✅ **Wikilink Conversion**: `[[note]]` → `[note](@/note.md)`  
✅ **Custom Link Text**: `[[note|display text]]` → `[display text](@/note.md)`  
✅ **Directory Navigation**: `[[folder/note]]` → `[folder/note](@/folder/note.md)`  
✅ **Image Processing**: `![[image.png]]` → `![image.png](image.png)`  
✅ **Static Asset Handling**: `![img](static/logo.png)` → `![img](/logo.png)`  
✅ **Markdown Embedding**: `![[snippet.md]]` → embedded content  
✅ **External Link Preservation**: URLs and external links remain unchanged  
✅ **Relative Path Resolution**: Proper path resolution from subdirectories  
✅ **Frontmatter Processing**: Maintains YAML frontmatter  
✅ **Unresolvable Link Handling**: `[[missing]]` → `*missing*` (italic text)  

## Installation

### From Source

```bash
git clone https://github.com/yourusername/obsidian-zola.git
cd obsidian-zola
cargo build --release
```

The binary will be available at `target/release/obsidian-zola`.

### Prerequisites

- Rust 1.70+ (uses latest obsidian-export)
- An Obsidian vault to convert
- A Zola site structure

## Usage

### Command Line

```bash
# Basic export
obsidian-zola export --source /path/to/obsidian/vault --destination /path/to/zola/content

# With verbose output
obsidian-zola export --source ./my-vault --destination ./my-site/content --verbose

# Skip frontmatter processing
obsidian-zola export --source ./vault --destination ./content --skip-frontmatter
```

### Library Usage

```rust
use obsidian_export::{Exporter, FrontmatterStrategy};
use obsidian_zola::postprocessors::create_zola_link_postprocessor;
use std::path::PathBuf;

let vault_path = PathBuf::from("path/to/vault");
let output_path = PathBuf::from("path/to/zola/content");

let mut exporter = Exporter::new(vault_path.clone(), output_path);
exporter.frontmatter_strategy(FrontmatterStrategy::Always);

let zola_postprocessor = create_zola_link_postprocessor(vault_path);
exporter.add_postprocessor(&zola_postprocessor);

exporter.run().expect("Export failed");
```

## How It Works

1. **Export Processing**: Uses `obsidian-export` to process the vault and resolve wikilinks
2. **Link Conversion**: Converts resolved markdown links to Zola's `@/` internal link format
3. **Image Processing**: Handles both wikilink images and regular markdown images
4. **Path Resolution**: Resolves relative paths correctly based on file location
5. **Static Assets**: Converts `static/` paths to root-relative paths for Zola

## Conversion Examples

### Wikilinks
```markdown
# Before (Obsidian)
[[Getting Started]]
[[tutorials/advanced|Advanced Guide]]
[[../index|Home]]

# After (Zola)
[Getting Started](@/getting-started.md)
[Advanced Guide](@/tutorials/advanced.md)
*Home*  # Unresolvable relative link becomes italic
```

### Images
```markdown
# Before (Obsidian)
![[diagram.png]]
![Logo](static/logo.png)
![Chart](../assets/chart.jpg)

# After (Zola)
![diagram.png](diagram.png)
![Logo](/logo.png)
![Chart](assets/chart.jpg)
```

## Limitations

- **Relative wikilinks**: `[[../note]]` may not resolve correctly (by design in obsidian-export)
- **Complex transclusions**: Only basic markdown embedding is supported
- **Plugin-specific syntax**: Obsidian plugin syntax is not processed
- **Binary files**: Only copies files, doesn't process binary formats

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test --test simple_test

# Run with output
cargo test -- --nocapture
```

### Project Structure

```
src/
├── main.rs              # CLI interface
├── lib.rs               # Library exports
├── postprocessors.rs    # Link conversion logic
└── utils.rs             # Utility functions
tests/
├── simple_test.rs       # Integration tests
└── test_vault/          # Test vault structure
```

## License

MIT License - see [LICENSE](LICENSE) file.

## Disclaimer

This software is provided "as is" without warranty of any kind. The author is not liable for any damages or data loss resulting from the use of this tool. Always backup your content before processing.

## Contributing

As this is a personal tool, contributions are not actively solicited. However, if you find bugs or have improvements that align with the tool's goals, feel free to open an issue or pull request.
