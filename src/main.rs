//! Zed theme converter command-line tool
//!
//! This binary provides a command-line interface for bidirectional conversion between VSCode and Zed theme formats.

use std::convert::From;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use thematic::*;

/// Convert between VSCode and Zed theme formats
#[derive(Parser)]
#[command(name = "zed-theme")]
#[command(about = "A CLI tool for bidirectional conversion between VSCode and Zed theme formats")]
#[command(long_about = "Converts between VSCode and Zed color theme formats. \
                        Supports conversion from VSCode themes to Zed format and vice versa. \
                        Output is written to stdout for easy piping and redirection.")]
#[command(version)]
pub struct Cli {
    // Conversion command
    //#[command(subcommand)]
    //pub command: Commands,
    /// Print little-to-no theme information.
    #[arg(global = true, short = 'q', default_value_t = false)]
    quiet: bool,
    /// Print more theme information.
    #[arg(global = true, short = 'v', default_value_t = false)]
    verbose: bool,
    /// The theme file to convert to or from Zed format
    #[arg(value_name = "INPUT_FILE")]
    input: PathBuf,
}

/// Available conversion commands
#[derive(Subcommand)]
pub enum Commands {
    /// Convert VSCode theme to Zed format
    #[command(name = "vscode-to-zed", alias = "vz")]
    #[command(about = "Convert a VSCode theme JSON file to Zed format")]
    VscodeToZed {
        /// Path to the VSCode theme JSON file
        #[arg(value_name = "INPUT_FILE")]
        #[arg(help = "VSCode theme JSON file to convert")]
        input: PathBuf,
    },
    /// Convert Zed theme to VSCode format
    #[command(name = "zed-to-vscode", alias = "zv")]
    #[command(about = "Convert a Zed theme JSON file to VSCode format")]
    ZedToVscode {
        /// Path to the Zed theme JSON file
        #[arg(value_name = "INPUT_FILE")]
        #[arg(help = "Zed theme JSON file to convert")]
        input: PathBuf,
    },
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

    match Theme::load(&cli.input)? {
        Theme::VSCode(t) => {
            log::info!("Converting VSCode theme {} to Zed...", cli.input.display());
            convert_vscode_to_zed(t);
        }
        Theme::Zed(t) => {
            log::info!("Converting Zed theme {} to VSCode...", cli.input.display());
            convert_zed_to_vscode(t);
        }
    }

    Ok(())
}

fn convert_vscode_to_zed(vscode_theme: VSCodeTheme) {
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

    let zed: ZedTheme = ZedTheme::from(&vscode_theme);
    let family = ZedThemeFamily {
        schema: Some("https://zed.dev/schema/themes/v0.2.0.json".to_string()),
        author: "".to_string(),
        name: zed.name.clone(),
        themes: vec![zed],
    };

    let Ok(content) = serde_json::to_string_pretty(&family) else {
        return;
    };
    println!("{content}");
    println!();
}

fn convert_zed_to_vscode(zed_theme_family: ZedThemeFamily) {
    log::info!("✓ Loaded Zed theme family: {}", zed_theme_family.name);
    log::debug!("Theme details:");
    log::debug!("  - Family name: {}", zed_theme_family.name);
    log::debug!("  - Author: {}", zed_theme_family.author);
    log::debug!("  - Themes in family: {}", zed_theme_family.themes.len());

    for (i, theme) in zed_theme_family.themes.iter().enumerate() {
        log::debug!("    {}. {} ({:?})", i + 1, theme.name, theme.appearance);
        let code: VSCodeTheme = theme.into();
        let Ok(content) = serde_json::to_string_pretty(&code) else {
            continue;
        };
        println!("{content}");
        println!();
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
