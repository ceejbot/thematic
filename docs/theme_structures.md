# Theme Data Structures

This document describes the Rust data structures created for handling VSCode and Zed color themes.

## Overview

We've implemented comprehensive data structures for both theme formats:

- **VSCode themes** (`src/vscode_theme.rs`) - Handles VSCode color theme JSON files
- **Zed themes** (`src/thematic.rs`) - Handles Zed theme JSON files according to v0.2.0 schema

Both structures use `serde_with` and the `#[skip_serializing_none]` macro for clean, minimal JSON serialization that omits `None` values.

## VSCode Theme Structure

The main structure is `VSCodeTheme` which contains:

- `name` - Theme display name
- `theme_type` - Optional type ("light" or "dark")
- `colors` - HashMap of workbench color definitions
- `token_colors` - Either a path to tmTheme file or array of token color rules
- `semantic_highlighting` - Boolean flag for semantic highlighting
- `semantic_token_colors` - Semantic token color definitions

### Key Features

- **Flexible token colors**: Supports both tmTheme file references and inline token rules
- **Helper methods**: Easy access to theme type checking and color retrieval
- **Comprehensive constants**: Pre-defined constants for common workbench colors and token scopes
- **Clean serialization**: Uses `serde_with` to automatically skip `None` fields in JSON output

## Zed Theme Structure

The main structure is `ZedThemeFamily` which contains:

- `schema` - Optional JSON schema reference
- `author` - Theme author information
- `name` - Theme family name
- `themes` - Array of individual themes

Each `ZedTheme` contains:

- `name` - Individual theme name
- `appearance` - Light or Dark enum
- `style` - Complete style definition with all color properties

### Key Features

- **Comprehensive coverage**: All documented Zed theme properties are supported
- **Structured organization**: Logical grouping of related color properties
- **Type safety**: Proper enums for appearance, font styles, and font weights
- **Default implementation**: Easy theme creation with sensible defaults
- **Clean serialization**: Uses `serde_with` to automatically skip `None` fields in JSON output

## Color Properties Coverage

The Zed theme structure includes all major color categories:

- **Base colors**: background, borders, surfaces
- **Element states**: hover, active, selected, disabled
- **UI components**: status bar, title bar, tabs, panels, editor
- **Terminal colors**: Complete ANSI color support
- **Status indicators**: error, warning, info, success states
- **Version control**: git status colors
- **Collaboration**: Player colors for multi-user editing
- **Syntax highlighting**: Flexible token-based system

## Usage Examples

### Command-Line Usage

The CLI provides bidirectional conversion between VSCode and Zed theme formats:

```bash
# Convert VSCode theme to Zed format (output to stdout)
zed-theme vscode-to-zed theme.json > converted-theme.json

# Convert Zed theme to VSCode format (output to stdout)
zed-theme zed-to-vscode zed-theme.json > converted-theme.json

# Show help
zed-theme --help
zed-theme vscode-to-zed --help
zed-theme zed-to-vscode --help
```

### Loading Themes (Library Usage)

```rust
use thematic::{load_vscode_theme, load_zed_theme};

// Load VSCode theme
let vscode_theme = load_vscode_theme("theme.json")?;
println!("Theme: {} ({})", vscode_theme.name,
         if vscode_theme.is_dark_theme() { "dark" } else { "light" });

// Load Zed theme family
let zed_family = load_zed_theme("zed-theme.json")?;
println!("Family: {} by {}", zed_family.name, zed_family.author);
```

### Creating New Themes

```rust
use thematic::vscode_theme::{VSCodeTheme, TokenColorRule, TokenColorSettings};

let mut theme = VSCodeTheme::new("My Theme".to_string());
theme.set_color("editor.background".to_string(), "#1e1e1e".to_string());
theme.add_token_rule(TokenColorRule::single_scope(
    "comment".to_string(),
    TokenColorSettings::foreground("#6A9955".to_string())
));
```

## Testing

The structures include comprehensive tests:

- **Fixture loading**: Validates against real theme files
- **Round-trip serialization**: Ensures data integrity
- **Helper methods**: Tests utility functions
- **Error handling**: Proper error types and handling

## File Organization

```
src
├── convert              # conversion implementations
│   ├── code_to_zed.rs
│   ├── mod.rs
│   └── zed_to_code.rs
├── errors.rs            # library error types
├── lib.rs							 # Library interface and utilities
├── main.rs              # CLI for converting themes
└── themes               # module for all supported theme types
    ├── mod.rs
    ├── vscode.rs        # VSCode theme data structures
    └── zed.rs           # Zed theme data structures
```

## Next Steps

These structures provide the foundation for:

1. **Theme conversion**: VSCode → Zed conversion logic
2. **Theme validation**: Ensuring themes meet format requirements
3. **Theme manipulation**: Programmatic theme editing and generation
4. **CLI tools**: Command-line theme conversion utilities

The structures are designed to be:
- **Extensible**: Easy to add new properties as schemas evolve
- **Type-safe**: Leveraging Rust's type system for correctness
- **Performant**: Efficient serialization/deserialization with clean JSON output
- **Well-documented**: Comprehensive documentation and examples
- **Clean**: Uses `serde_with` to eliminate repetitive `skip_serializing_if` annotations
