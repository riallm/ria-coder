//! Ria Coder CLI
//!
//! Terminal-based agentic coding system

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "ria")]
#[command(about = "Ria Coder - Terminal-based agentic coding system", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive coding session
    Coder {
        /// Path to GGUF model file
        #[arg(short, long)]
        model: Option<String>,

        /// Project root directory
        #[arg(short, long)]
        root: Option<String>,

        /// Theme name
        #[arg(long, default_value = "default")]
        theme: String,

        /// Enable vim mode
        #[arg(long)]
        vim: bool,
    },

    /// Generate code from prompt
    Generate {
        /// Path to GGUF model file
        #[arg(short, long)]
        model: Option<String>,

        /// Prompt text
        #[arg(short, long)]
        prompt: String,

        /// Max tokens to generate
        #[arg(long, default_value = "256")]
        max_tokens: usize,

        /// Temperature
        #[arg(long, default_value = "0.7")]
        temperature: f64,
    },

    /// Inspect a GGUF model file
    Inspect {
        /// Path to GGUF file
        #[arg(short, long)]
        model: String,
    },

    /// Download a model
    Download {
        /// Model name (e.g., ria-8b-q4_k_m)
        model: String,
    },

    /// List available models
    Models,

    /// Show configuration
    Config {
        /// Show config path
        #[arg(long)]
        path: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ria=info".parse()?)
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Coder { model, root, theme, vim } => {
            cmd_coder(model.as_deref(), root.as_deref(), &theme, vim).await?;
        }
        Commands::Generate { model, prompt, max_tokens, temperature } => {
            cmd_generate(model.as_deref(), &prompt, max_tokens, temperature).await?;
        }
        Commands::Inspect { model } => {
            cmd_inspect(&model)?;
        }
        Commands::Download { model } => {
            cmd_download(&model)?;
        }
        Commands::Models => {
            cmd_models()?;
        }
        Commands::Config { path } => {
            cmd_config(path)?;
        }
    }

    Ok(())
}

/// Start interactive coding session
async fn cmd_coder(
    model: Option<&str>,
    root: Option<&str>,
    theme: &str,
    vim: bool,
) -> Result<()> {
    println!("⚡ Ria Coder v{}", env!("CARGO_PKG_VERSION"));
    println!("📁 Project: {}", root.unwrap_or("."));
    println!("🤖 Model: {}", model.unwrap_or("ria-8b-q4_k_m.gguf"));
    println!("🎨 Theme: {}", theme);
    if vim { println!("⌨️  Vim mode: enabled"); }
    println!();
    println!("Starting interactive session...");

    // Initialize TUI
    // Load model via riallm
    // Start main loop

    Ok(())
}

/// Generate code from prompt
async fn cmd_generate(
    model: Option<&str>,
    prompt: &str,
    max_tokens: usize,
    temperature: f64,
) -> Result<()> {
    println!("🤖 Generating with {} (temp={})...", 
        model.unwrap_or("ria-8b"), temperature);
    println!("Prompt: {}", prompt);

    // Load model
    // Generate response
    // Print output

    Ok(())
}

/// Inspect GGUF model
fn cmd_inspect(model: &str) -> Result<()> {
    println!("📋 Inspecting: {}", model);
    // Use ria-gguf to parse and display metadata
    Ok(())
}

/// Download model
fn cmd_download(model: &str) -> Result<()> {
    println!("⬇️  Downloading: {}", model);
    // Download from registry
    Ok(())
}

/// List available models
fn cmd_models() -> Result<()> {
    println!("📦 Available models:");
    println!("  ria-1b-q4_k_m     (0.6 GB, edge devices)");
    println!("  ria-8b-q4_k_m     (4.9 GB, recommended)");
    println!("  ria-64b-q4_k_m    (37 GB, complex tasks)");
    println!("  ria-128b-q4_k_m   (74 GB, enterprise)");
    Ok(())
}

/// Show configuration
fn cmd_config(show_path: bool) -> Result<()> {
    let config = ria_config::Config::default();
    if show_path {
        println!("Config path: {:?}", ria_config::Config::default_path());
    } else {
        println!("{}", toml::to_string_pretty(&config)?);
    }
    Ok(())
}
