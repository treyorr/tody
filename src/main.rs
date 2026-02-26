use anyhow::Result;
use clap::{Parser, Subcommand};
use demand::DemandOption;
use tody::config::{AppConfig, DefaultView};
use tody::db::{Database, ListFilter, ScopeFilter, resolve_local_folder_path};
use tody::ui;

#[derive(Parser)]
#[command(
    name = "tody",
    version,
    about = "Tiny and tidy global/local path-aware task manager"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Add a new task
    Add {
        /// Task title
        title: String,
        /// Store under the current project folder
        #[arg(short, long)]
        local: bool,
    },
    /// Print tasks to stdout
    List {
        /// Show only global tasks
        #[arg(short, long)]
        global: bool,
        /// Show only local tasks for the current folder
        #[arg(short, long)]
        local: bool,
        /// Include completed tasks
        #[arg(short, long)]
        all: bool,
    },
    /// Mark a task as completed
    Done {
        /// Task id
        id: i64,
    },
    /// Hard delete a task
    Rm {
        /// Task id
        id: i64,
    },
    /// Show recently completed tasks
    Log,
    /// Find orphaned folder paths and optionally delete those tasks
    Prune,
    /// Update tody to the latest GitHub release
    Update,
    /// Delete database, config, and executable
    Uninstall,
    /// Read or write configuration values
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Set a config key
    Set {
        /// Key name (default_view | color_local | color_global)
        key: String,
        /// Value to write
        value: String,
    },
    /// Get a config key
    Get {
        /// Key name (default_view | color_local | color_global)
        key: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("  \x1b[31merror:\x1b[0m {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => cmd_interactive(),
        Some(Command::Add { title, local }) => cmd_add(&title, local),
        Some(Command::List { global, local, all }) => cmd_list(global, local, all),
        Some(Command::Done { id }) => cmd_done(id),
        Some(Command::Rm { id }) => cmd_rm(id),
        Some(Command::Log) => cmd_log(),
        Some(Command::Prune) => cmd_prune(),
        Some(Command::Update) => tody::update::run_update(env!("CARGO_PKG_VERSION")),
        Some(Command::Uninstall) => tody::uninstall::run_uninstall(),
        Some(Command::Config { action }) => match action {
            ConfigAction::Set { key, value } => cmd_config_set(&key, &value),
            ConfigAction::Get { key } => cmd_config_get(&key),
        },
    }
}

// ─── Helpers ────────────────────────────────────────────────────────

fn confirm(prompt: impl Into<String>) -> Result<bool> {
    demand::Confirm::new(prompt)
        .affirmative("Yes")
        .negative("No")
        .run()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::Interrupted => anyhow::anyhow!("cancelled"),
            _ => anyhow::anyhow!("prompt error: {e}"),
        })
}

// ─── Commands ───────────────────────────────────────────────────────

fn cmd_interactive() -> Result<()> {
    let config = AppConfig::load_or_default()?;
    let db = Database::open_default()?;

    let scope = match config.default_view {
        DefaultView::Merged => ScopeFilter::Merged,
        DefaultView::Local => ScopeFilter::LocalOnly,
        DefaultView::Global => ScopeFilter::GlobalOnly,
    };

    let tasks = db.list_tasks(ListFilter {
        scope,
        include_completed: false,
    })?;
    if tasks.is_empty() {
        println!();
        println!("  \x1b[2mNo pending tasks.\x1b[0m");
        println!("  \x1b[2mRun\x1b[0m tody add \"your task\" \x1b[2mto get started.\x1b[0m");
        return Ok(());
    }

    let options: Vec<DemandOption<i64>> = tasks
        .iter()
        .map(|t| DemandOption::new(t.id).label(&ui::format_task_option(t, &config)))
        .collect();

    let task_id = demand::Select::new("Mark a task as done")
        .description("Select a task to complete")
        .filterable(true)
        .options(options)
        .run()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::Interrupted => anyhow::anyhow!("cancelled"),
            _ => anyhow::anyhow!("interactive selection failed: {e}"),
        })?;

    db.mark_done(task_id)?;
    let title = tasks
        .iter()
        .find(|t| t.id == task_id)
        .map(|t| t.title.as_str())
        .unwrap_or("task");
    println!("  \x1b[32m✓\x1b[0m Completed: {title}");
    Ok(())
}

fn cmd_add(title: &str, local: bool) -> Result<()> {
    let db = Database::open_default()?;
    let folder_path = if local {
        Some(resolve_local_folder_path()?)
    } else {
        None
    };
    let id = db.add_task(title, folder_path.as_deref())?;

    let scope = if local {
        "\x1b[95m\x1b[3mlocal\x1b[0m"
    } else {
        "\x1b[96m\x1b[3mglobal\x1b[0m"
    };
    println!("  \x1b[32m✓\x1b[0m {scope} task #{id}: {title}");
    Ok(())
}

fn cmd_list(global: bool, local: bool, all: bool) -> Result<()> {
    let config = AppConfig::load_or_default()?;
    let db = Database::open_default()?;

    let scope = if global {
        ScopeFilter::GlobalOnly
    } else if local {
        ScopeFilter::LocalOnly
    } else {
        match config.default_view {
            DefaultView::Merged => ScopeFilter::Merged,
            DefaultView::Local => ScopeFilter::LocalOnly,
            DefaultView::Global => ScopeFilter::GlobalOnly,
        }
    };

    let tasks = db.list_tasks(ListFilter {
        scope,
        include_completed: all,
    })?;
    ui::print_header("Tasks");
    ui::print_task_table(&tasks, &config);
    println!();
    Ok(())
}

fn cmd_done(id: i64) -> Result<()> {
    Database::open_default()?.mark_done(id)?;
    println!("  \x1b[32m✓\x1b[0m Task #{id} marked as completed.");
    Ok(())
}

fn cmd_rm(id: i64) -> Result<()> {
    if !confirm(format!("Delete task #{id}?"))? {
        println!("  Cancelled.");
        return Ok(());
    }
    Database::open_default()?.remove_task(id)?;
    println!("  \x1b[32m✓\x1b[0m Task #{id} deleted.");
    Ok(())
}

fn cmd_log() -> Result<()> {
    let config = AppConfig::load_or_default()?;
    let tasks = Database::open_default()?.recent_completed(25)?;
    ui::print_header("Recently Completed");
    ui::print_log(&tasks, &config);
    println!();
    Ok(())
}

fn cmd_prune() -> Result<()> {
    let mut db = Database::open_default()?;
    let orphans = db.orphaned_folder_paths()?;

    if orphans.is_empty() {
        println!("  \x1b[2mNo orphaned folder paths found. All clean!\x1b[0m");
        return Ok(());
    }

    println!("  Found {} orphaned folder path(s):", orphans.len());
    for path in &orphans {
        println!("    \x1b[33m•\x1b[0m {}", path.display());
    }
    println!();

    if !confirm("Delete all tasks for these missing folders?")? {
        println!("  Cancelled.");
        return Ok(());
    }

    let deleted = db.delete_tasks_for_folder_paths(&orphans)?;
    println!("  \x1b[32m✓\x1b[0m Pruned {deleted} orphaned task(s).");
    Ok(())
}

fn cmd_config_set(key: &str, value: &str) -> Result<()> {
    let mut config = AppConfig::load_or_default()?;
    config.set_key(key, value)?;
    config.save()?;
    println!("  \x1b[32m✓\x1b[0m Set {key} = {value}");
    Ok(())
}

fn cmd_config_get(key: &str) -> Result<()> {
    let config = AppConfig::load_or_default()?;
    println!("  {key} = {}", config.get_key(key)?);
    Ok(())
}
