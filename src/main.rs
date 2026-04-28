mod analyzer;
mod code_index;
mod config;
mod error;
mod git;
mod llm;
mod summarizer;
mod templates;

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::info;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "code-summarizer")]
#[command(author, version, about = "Generates hierarchical, AI-optimized markdown summaries of codebase architecture")]
struct Cli {
    /// Project root directory (defaults to current directory)
    #[arg(long, global = true)]
    project_root: Option<PathBuf>,

    /// Output directory for generated summaries
    #[arg(long, short, global = true, default_value = ".ai/context")]
    output: PathBuf,

    /// Verbose output
    #[arg(long, short, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Quiet mode (suppress non-error output)
    #[arg(long, short, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate all summaries (architecture, modules, dependencies, hotspots)
    Generate {
        /// Use a specific template file
        #[arg(long)]
        template: Option<PathBuf>,

        /// Use local LLM for descriptions (e.g., "ollama")
        #[arg(long)]
        llm: Option<String>,

        /// LLM model to use
        #[arg(long, default_value = "llama3.3")]
        model: String,

        /// Custom LLM endpoint
        #[arg(long)]
        llm_endpoint: Option<String>,

        /// Disable LLM (structural analysis only)
        #[arg(long)]
        no_llm: bool,
    },

    /// Update stale summaries
    Update {
        /// Only update if summaries are stale
        #[arg(long)]
        if_stale: bool,

        /// Force regeneration even if not stale
        #[arg(long)]
        force: bool,

        /// Update specific summary type
        #[arg(long, value_parser = ["architecture", "modules", "dependencies", "hotspots"])]
        r#type: Option<String>,
    },

    /// Generate summary for a specific module
    Module {
        /// Path to the module directory
        #[arg(long)]
        path: PathBuf,

        /// Detail level for the summary
        #[arg(long, default_value = "standard", value_parser = ["minimal", "standard", "detailed"])]
        depth: String,
    },

    /// Show project statistics
    Stats,

    /// Show dependency graph
    Deps {
        /// Output format
        #[arg(long, default_value = "ascii", value_parser = ["ascii", "json"])]
        format: String,
    },

    /// List hot files (complex/frequently changed)
    Hotspots {
        /// Number of hotspots to show
        #[arg(long, default_value = "10")]
        limit: usize,

        /// Sort by metric
        #[arg(long, default_value = "hotness", value_parser = ["hotness", "complexity", "changes"])]
        sort_by: String,
    },

    /// Template management
    Templates {
        #[command(subcommand)]
        action: TemplateAction,
    },
}

#[derive(Subcommand)]
enum TemplateAction {
    /// List available templates
    List,

    /// Create a new template
    Create {
        /// Name for the new template
        #[arg(long)]
        name: String,
    },

    /// Validate a template file
    Validate {
        /// Template file to validate
        #[arg(long)]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    let log_level = if cli.quiet {
        "error"
    } else {
        match cli.verbose {
            0 => "info",
            1 => "debug",
            _ => "trace",
        }
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    let project_root = cli
        .project_root
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    info!("Project root: {:?}", project_root);
    info!("Output directory: {:?}", cli.output);

    match cli.command {
        Commands::Generate {
            template,
            llm,
            model,
            llm_endpoint,
            no_llm,
        } => {
            summarizer::generate_all(&project_root, &cli.output, template, llm, model, llm_endpoint, no_llm)?;
        }
        Commands::Update {
            if_stale,
            force,
            r#type,
        } => {
            summarizer::update_summaries(&project_root, &cli.output, if_stale, force, r#type)?;
        }
        Commands::Module { path, depth } => {
            summarizer::generate_module(&project_root, &cli.output, &path, &depth)?;
        }
        Commands::Stats => {
            analyzer::show_stats(&project_root)?;
        }
        Commands::Deps { format } => {
            analyzer::show_deps(&project_root, &format)?;
        }
        Commands::Hotspots { limit, sort_by } => {
            analyzer::show_hotspots(&project_root, limit, &sort_by)?;
        }
        Commands::Templates { action } => match action {
            TemplateAction::List => {
                templates::list_templates()?;
            }
            TemplateAction::Create { name } => {
                templates::create_template(&name)?;
            }
            TemplateAction::Validate { file } => {
                templates::validate_template(&file)?;
            }
        },
    }

    Ok(())
}
