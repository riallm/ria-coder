//! Ria Coder CLI
//!
//! Terminal-based agentic coding system

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

        /// Project root directory
        #[arg(short, long)]
        root: Option<String>,

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
            tracing_subscriber::EnvFilter::from_default_env().add_directive("ria=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Coder {
            model,
            root,
            theme,
            vim,
        } => {
            cmd_coder(model.as_deref(), root.as_deref(), &theme, vim).await?;
        }
        Commands::Generate {
            model,
            root,
            prompt,
            max_tokens,
            temperature,
        } => {
            cmd_generate(
                model.as_deref(),
                root.as_deref(),
                &prompt,
                max_tokens,
                temperature,
            )
            .await?;
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
async fn cmd_coder(model: Option<&str>, root: Option<&str>, theme: &str, vim: bool) -> Result<()> {
    use ria_agent::orchestrator::AgentOrchestrator;
    use ria_tui::{App, Theme};
    let config = load_config();

    let project_root = match root {
        Some(r) => PathBuf::from(r),
        None => std::env::current_dir()?,
    };
    let model_path = model.or(config.model.path.as_deref());

    let _theme_name = if theme == "default" {
        config.ui.theme.as_str()
    } else {
        theme
    };
    let theme = Theme::default();
    let llm = load_llm(model_path).await?;
    let mut orchestrator = AgentOrchestrator::new(llm);
    orchestrator.max_iterations = config.agent.max_iterations;

    let mut app = App::new(theme, project_root, orchestrator)?;
    app.keybindings.vim_mode = vim;

    app.run()?;

    Ok(())
}

/// Generate code from prompt
async fn cmd_generate(
    model: Option<&str>,
    root: Option<&str>,
    prompt: &str,
    max_tokens: usize,
    temperature: f64,
) -> Result<()> {
    use ria_agent::llm::GenConfig;
    use ria_agent::orchestrator::AgentOrchestrator;
    let config = load_config();

    let project_root = match root {
        Some(r) => PathBuf::from(r),
        None => std::env::current_dir()?,
    };
    let model_path = model.or(config.model.path.as_deref());

    println!(
        "🤖 Generating with {} (temp={}, max_tokens={})...",
        model_path.unwrap_or("mock-ria-8b"),
        temperature,
        max_tokens
    );
    println!("📁 Project: {:?}", project_root);
    println!("Prompt: {}", prompt);
    println!();

    let llm = load_llm(model_path).await?;
    let mut orchestrator = AgentOrchestrator::new(llm);
    orchestrator.max_iterations = config.agent.max_iterations;
    orchestrator.init(project_root)?;

    if prompt.starts_with("raw:") {
        let output = orchestrator
            .llm
            .generate(
                prompt.trim_start_matches("raw:").trim(),
                &GenConfig {
                    max_tokens,
                    temperature,
                    ..Default::default()
                },
            )
            .await?;
        println!("{}", output);
        return Ok(());
    }

    let output = orchestrator.process_request(prompt).await?;

    println!("---");
    println!("{}", output.message);
    println!("---");
    println!(
        "Changes: {}, Tests passed: {:?}",
        output.changes_made, output.tests_passed
    );

    Ok(())
}

/// Inspect GGUF model
fn cmd_inspect(model: &str) -> Result<()> {
    let path = PathBuf::from(model);
    let metadata = std::fs::metadata(&path)?;
    println!("📋 Inspecting: {}", path.display());
    println!("Size: {:.2} MB", metadata.len() as f64 / 1_048_576.0);
    println!("Readonly: {}", metadata.permissions().readonly());
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

fn load_config() -> ria_config::Config {
    let path = ria_config::Config::default_path();
    ria_config::Config::load(&path).unwrap_or_default()
}

async fn load_llm(model: Option<&str>) -> Result<Box<dyn ria_agent::llm::LLMEngine>> {
    match model {
        Some(model_path) => Ok(Box::new(
            ria_agent::llm::RiaLLMEngine::new(model_path).await?,
        )),
        None => Ok(Box::new(ria_agent::llm::MockLLMEngine::new())),
    }
}
