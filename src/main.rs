//! Zed theme converter command-line tool
//!
//! This binary provides a command-line interface for bidirectional conversion between
//! VSCode and Zed theme formats. It can parse Zed theme families and write out minimum
//! viable Zed theme extensions. It can do the same for VSCode themes.

use std::convert::From;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use clap::{Parser, Subcommand};
use thematic::themes::{Extension, ThemeFile, VsCodeExtension, ZedExtension};
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
    /// Print little-to-no theme information.
    #[arg(global = true, short = 'q', default_value_t = false)]
    quiet: bool,
    /// Print more theme information.
    #[arg(global = true, short = 'v', default_value_t = false)]
    verbose: bool,
    /// The conversion command.
    #[command(subcommand)]
    command: Convert,
}

/// Available conversion commands
#[derive(Subcommand)]
pub enum Convert {
    /// Treat the innput as a VSCode theme and convert it to Zed format
    #[command(name = "vscode-to-zed", alias = "vz")]
    #[command(about = "Convert a VSCode theme JSON file to Zed format")]
    VscodeToZed {
        /// The theme file to convert to Zed format. If left out, input
        /// is read from stdin and output is written to stdout.
        #[arg(value_name = "/path/to/theme")]
        input: Option<PathBuf>,
    },
    /// Treat the input as a Zed theme and convert it tox VSCode format
    #[command(name = "zed-to-vscode", alias = "zv")]
    #[command(about = "Convert a Zed theme JSON file to VSCode format")]
    ZedToVscode {
        /// The theme file to convert to VSCode format. If left out, input
        /// is read from stdin and output is written to stdout.
        #[arg(value_name = "/path/to/theme")]
        input: Option<PathBuf>,
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
            if let Some(fname) = input {
                // todo see if directory is an extension
                let data = read_file(&fname)?;
                let theme = VsCodeTheme::from_bytes(data.as_slice())?;
                let converted = convert_vscode_to_zed(theme);
                converted.write()?;
            } else {
                let bytes = read_stdin()?;
                let theme = VsCodeTheme::from_bytes(bytes.as_slice())?;
                let zed = ZedTheme::from(&theme);
                println!("{}", serde_json::to_string_pretty(&zed)?);
            }
        }
        Convert::ZedToVscode { input } => {
            if let Some(fname) = input {
                // todo see if directory is an extension
                let data = read_file(&fname)?;
                let theme = ZedThemeFamily::from_bytes(data.as_slice())?;
                // print a lot only in this case
                let converted = convert_zed_to_vscode(theme);
                converted.write()?;
            } else {
                let bytes = read_stdin()?;
                let family = ZedThemeFamily::from_bytes(bytes.as_slice())?;
                for theme in family.themes {
                    let vscode_theme = VsCodeTheme::from(&theme);
                    println!("{}", serde_json::to_string_pretty(&vscode_theme)?);
                }
            }
        }
    }

    Ok(())
}

fn read_stdin() -> Result<Vec<u8>, ThemeError> {
    let mut data: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(std::io::stdin());
    log::debug!("Reading from stdin...");
    let count = reader.read(&mut data)?;

    if count == 0 {
        log::warn!("No theme data to convert!");
        std::process::exit(1);
    }

    Ok(data)
}

fn read_file(fname: &PathBuf) -> Result<Vec<u8>, ThemeError> {
    let mut data: Vec<u8> = Vec::new();
    log::debug!("Reading from file '{}'...", fname.display());
    let mut fp = std::fs::File::open(fname)?;

    let count = fp.read_to_end(&mut data)?;
    if count == 0 {
        log::warn!("No theme data to convert!");
        std::process::exit(1);
    }

    Ok(data)
}

fn convert_vscode_to_zed(vscode_theme: VsCodeTheme) -> ZedExtension {
    log::info!("✓ Loaded VSCode theme: {}", vscode_theme.name);
    log::debug!("Theme details:");
    log::debug!("  - Name: {}", vscode_theme.name);
    if let Some(theme_type) = vscode_theme.get_theme_type() {
        log::debug!("  - Type: {}", theme_type);
    }
    log::debug!("  - Is dark theme: {}", vscode_theme.is_dark_theme());

    if let Some(colors) = &vscode_theme.colors {
        log::debug!("  - Workbench colors: {}", colors.len());
    }

    if let Some(token_rules) = vscode_theme.get_token_rules() {
        log::debug!("  - Token color rules: {}", token_rules.len());
    }

    ZedExtension::from(&vscode_theme)
}

fn convert_zed_to_vscode(zed_theme_family: ZedThemeFamily) -> VsCodeExtension {
    log::info!("✓ Loaded Zed theme family: {}", zed_theme_family.name);
    log::debug!("Theme details:");
    log::debug!("  - Family name: {}", zed_theme_family.name);
    log::debug!("  - Author: {}", zed_theme_family.author);
    log::debug!("  - Themes in family: {}", zed_theme_family.themes.len());

    VsCodeExtension::from(&zed_theme_family)
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
