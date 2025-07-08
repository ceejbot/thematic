//! Zed theme converter command-line tool
//!
//! This binary provides a command-line interface for bidirectional conversion between
//! VSCode and Zed theme formats. It can parse Zed theme families and write out minimum
//! viable Zed theme extensions. It can do the same for VSCode themes.

use std::convert::From;

use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use thematic::editors::{Extension, VsCodeExtension, ZedExtension};
use thematic::*;

/// Convert between VSCode and Zed theme formats
#[derive(Parser)]
#[command(name = "thematic", styles = v3_styles(), version)]
#[command(about = "A CLI tool for bidirectional conversion between VSCode and Zed theme formats")]
#[command(
    max_term_width = 100,
    long_about = "Converts between VSCode and Zed color theme formats. \
        Supports conversion from VSCode themes to Zed format and vice versa. \
        Output is written to the destination editor's default theme location."
)]
#[command(version)]
pub struct Cli {
    /// Quiet output.
    #[arg(global = true, short = 'q', default_value_t = false)]
    quiet: bool,
    /// Output with theme information.
    #[arg(global = true, short = 'v', default_value_t = false)]
    verbose: bool,
    /// The conversion or list command.
    #[command(subcommand)]
    command: Convert,
}

/// Available conversion commands
#[derive(Subcommand)]
pub enum Convert {
    /// Convert a VSCode theme JSON file to Zed format; `vz` for short
    #[command(name = "vscode-to-zed", alias = "vz")]
    VscodeToZed {
        /// The name or part of the name of a VSCode theme to convert to Zed format
        #[arg(value_name = "theme-name")]
        input: String,
    },
    /// Convert a Zed theme JSON file to VSCode format; `zv` for short
    #[command(name = "zed-to-vscode", alias = "zv")]
    ZedToVscode {
        /// The name or part of the name of a Zed theme to convert to VSCode format
        #[arg(value_name = "theme-name")]
        input: String,
    },
    /// Find all the Zed themes with names matching the input pattern; `zed` for short
    #[command(alias = "zed")]
    ZedList {
        /// the string to search for
        pattern: String,
    },
    /// Find all the VSCode themes with names matching the input pattern; `vsc` for short
    #[command(alias = "vsc")]
    VSCodeList {
        /// the string to search for
        pattern: String,
    },
}

fn v3_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Yellow.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}

fn main() -> Result<(), ThemeError> {
    let cli = Cli::parse();
    let level = if cli.quiet {
        log::LevelFilter::Warn
    } else if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    let config = lovely_env_logger::Config {
        with_system_timestamp: false,
        reltime: false,
        short_levels: false,
        with_file_name: false,
        with_line_number: false,
        with_padding: true,
    };
    lovely_env_logger::formatted_builder(config).filter(None, level).init();

    match cli.command {
        Convert::VscodeToZed { input } => {
            handle_vscode_extension(input)?;
        }
        Convert::ZedToVscode { input } => {
            handle_zed_extension(input)?;
        }
        Convert::ZedList { pattern } => {
            handle_zed_search(pattern)?;
        }
        Convert::VSCodeList { pattern } => {
            handle_vscode_search(pattern)?;
        }
    }

    Ok(())
}

fn handle_vscode_extension(fname: String) -> Result<(), ThemeError> {
    let vscode = *(VsCodeExtension::read(fname.as_str())?);

    log::info!("✓ Loaded VSCode theme extension: {fname}");
    log::debug!("Theme details:");
    log::debug!("  - Name: {}", vscode.manifest().name());
    log::debug!("  - Color themes in extension: {}", vscode.manifest().themes().len());
    log::debug!(
        "  - Icon themes in extension: {}",
        vscode.manifest().icon_themes().len()
    );

    let converted = ZedExtension::from(vscode);
    converted.write()?;
    // add it to the list of installed Zed extensions
    ZedExtension::add_installed(&converted)?;

    // Provide summary of what was converted
    let color_count = converted.families().iter().map(|f| f.themes.len()).sum::<usize>();
    let icon_count = converted.icon_themes().len();

    if color_count > 0 && icon_count > 0 {
        log::info!("✓ Converted {color_count} color theme(s) and {icon_count} icon theme(s) to Zed format.");
    } else if color_count > 0 {
        log::info!("✓ Converted {color_count} color theme(s) to Zed format.");
    } else if icon_count > 0 {
        log::info!("✓ Converted {icon_count} icon theme(s) to Zed format.");
    } else {
        log::warn!("No themes found to convert.");
    }

    Ok(())
}

fn handle_zed_extension(fname: String) -> Result<(), ThemeError> {
    let zed = *(ZedExtension::read(fname.as_str())?);

    // print a lot only in this case
    log::info!("✓ Loaded Zed theme extension: {fname}");
    log::debug!("Theme details:");
    log::debug!("  - Name: {}", zed.manifest().name());
    log::debug!("  - Authors: {:#?}", zed.manifest().authors());
    log::debug!("  - Color themes in extension: {}", zed.manifest().themes().len());
    log::debug!("  - Icon themes in extension: {}", zed.manifest().icon_themes().len());

    let converted = VsCodeExtension::from(zed);
    converted.write()?;

    // Provide summary of what was converted
    let color_count = converted.themes().len();
    let icon_count = converted.icon_themes().len();

    if color_count > 0 && icon_count > 0 {
        log::info!("✓ Converted {color_count} color theme(s) and {icon_count} icon theme(s) to VSCode format.");
    } else if color_count > 0 {
        log::info!("✓ Converted {color_count} color theme(s) to VSCode format.");
    } else if icon_count > 0 {
        log::info!("✓ Converted {icon_count} icon theme(s) to VSCode format.");
    } else {
        log::warn!("No themes found to convert.");
    }

    Ok(())
}

fn handle_zed_search(pattern: String) -> Result<(), ThemeError> {
    // Find all zed extensions matching the pattern (literally if it compiles to a glob)
    let matches = ZedExtension::search(pattern.as_str())?;
    println!();
    println!("Found {}", pluralize(matches.len(), "match", "matches"));
    println!();
    for found in matches {
        println!("• {}", found.name().yellow());
        println!(
            "    {}",
            pluralize(found.manifest().themes().len(), "color theme", "color themes")
        );
        println!(
            "    {}",
            pluralize(found.manifest().icon_themes().len(), "icon theme", "icon themes")
        );
        println!("    {}", found.official_path().display());
    }
    Ok(())
}

fn handle_vscode_search(pattern: String) -> Result<(), ThemeError> {
    let matches = VsCodeExtension::search(pattern.as_str())?;
    println!();
    println!("Found {}", pluralize(matches.len(), "match", "matches"));
    println!();
    for found in matches {
        println!("• {}", found.name().yellow());
        println!(
            "    {}",
            pluralize(found.manifest().themes().len(), "color theme", "color themes")
        );
        println!(
            "    {}",
            pluralize(found.manifest().icon_themes().len(), "icon theme", "icon themes")
        );
        println!("    {}", found.official_path().display());
    }
    Ok(())
}

fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    if count == 0 {
        format!("{} {plural}", "no".bold().blue())
    } else if count == 1 {
        format!("{} {singular}", "one".bold().blue())
    } else {
        format!("{} {plural}", count.bold().blue())
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert()
    }
}
