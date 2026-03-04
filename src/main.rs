use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use demand::DemandOption;
use tody::config::{AppConfig, DefaultView};
use tody::db::{Database, ListFilter, ScopeFilter, StatusFilter, project_name, try_resolve_project_path};
use tody::ui;

#[derive(Parser)]
#[command(
    name = "tody",
    version,
    about = "Tiny and tidy path-aware task manager for developers",
    after_help = "Inside a git repo, tasks are project-scoped by default.\nUse -g/--global to work with global tasks instead."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Add a new task (project-local by default inside a git repo)
    Add {
        /// Task title
        title: String,
        /// Store as a global task (overrides project auto-detection)
        #[arg(short, long)]
        global: bool,
        /// Store as a project-local task (default when in a git repo)
        #[arg(short, long)]
        local: bool,
    },
    /// List tasks (project-scoped by default inside a git repo)
    List {
        /// Show only global tasks
        #[arg(short, long)]
        global: bool,
        /// Show only local project tasks
        #[arg(short, long)]
        local: bool,
        /// Include everything across all projects
        #[arg(short, long)]
        all: bool,
        /// Show completed tasks instead of pending
        #[arg(short = 'd', long)]
        done: bool,
    },
    /// Mark a task as completed
    Done {
        /// Task id
        id: i64,
    },
    /// Edit a task's title
    Edit {
        /// Task id
        id: i64,
        /// New title
        title: String,
    },
    /// Undo the last completed task
    Undo,
    /// Hard delete a task
    Rm {
        /// Task id
        id: i64,
    },
    /// Show recently completed tasks
    Log {
        /// Show log from all projects
        #[arg(short, long)]
        all: bool,
    },
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
        Some(Command::Add {
            title,
            global,
            local,
        }) => cmd_add(&title, global, local),
        Some(Command::List {
            global,
            local,
            all,
            done,
        }) => cmd_list(global, local, all, done),
        Some(Command::Done { id }) => cmd_done(id),
        Some(Command::Edit { id, title }) => cmd_edit(id, &title),
        Some(Command::Undo) => cmd_undo(),
        Some(Command::Rm { id }) => cmd_rm(id),
        Some(Command::Log { all }) => cmd_log(all),
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

/// Resolve project context: (folder_path_string, is_in_project).
fn project_context() -> (Option<String>, bool) {
    match try_resolve_project_path() {
        Some(p) => (Some(p.to_string_lossy().to_string()), true),
        None => (None, false),
    }
}

// ─── Commands ───────────────────────────────────────────────────────

fn cmd_interactive() -> Result<()> {
    let config = AppConfig::load_or_default()?;
    let db = Database::open_default()?;
    let (project_folder, in_project) = project_context();

    // Resolve scope based on context + config
    let scope = match config.default_view {
        DefaultView::Auto => {
            if in_project {
                ScopeFilter::LocalCurrent
            } else {
                ScopeFilter::GlobalOnly
            }
        }
        DefaultView::Merged => ScopeFilter::MergedCurrent,
        DefaultView::Local => ScopeFilter::LocalCurrent,
        DefaultView::Global => ScopeFilter::GlobalOnly,
    };

    let tasks = db.list_tasks(ListFilter {
        scope,
        status: StatusFilter::PendingOnly,
        current_local_folder: project_folder.clone(),
    })?;

    if tasks.is_empty() {
        println!();
        if in_project {
            let name = project_folder
                .as_deref()
                .map(|p| project_name(std::path::Path::new(p)))
                .unwrap_or_default();
            println!("  \x1b[2mNo pending tasks in\x1b[0m {name}\x1b[2m.\x1b[0m");
        } else {
            println!("  \x1b[2mNo pending global tasks.\x1b[0m");
        }
        println!("  \x1b[2mRun\x1b[0m tody add \"your task\" \x1b[2mto get started.\x1b[0m");
        return Ok(());
    }

    let label: String = if in_project {
        let name = project_folder
            .as_deref()
            .map(|p| project_name(std::path::Path::new(p)))
            .unwrap_or_default();
        format!("Complete a task · {name}")
    } else {
        "Complete a task".into()
    };

    let options: Vec<DemandOption<i64>> = tasks
        .iter()
        .map(|t| DemandOption::new(t.id).label(&ui::format_task_option(t, &config)))
        .collect();

    let task_id = demand::Select::new(label.as_str())
        .description("Select a task to mark as done")
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

fn cmd_add(title: &str, global: bool, local: bool) -> Result<()> {
    if global && local {
        bail!("cannot use both --global and --local");
    }

    let db = Database::open_default()?;
    let project_path = try_resolve_project_path();
    let in_project = project_path.is_some();

    // Default: local if in a project, global otherwise
    let is_local = if global {
        false
    } else if local {
        if !in_project {
            bail!("--local requires being inside a git repository");
        }
        true
    } else {
        in_project
    };

    let folder_path = if is_local { project_path } else { None };
    let id = db.add_task(title, folder_path.as_deref())?;

    let scope = if is_local {
        "\x1b[95m\x1b[3mlocal\x1b[0m"
    } else {
        "\x1b[96m\x1b[3mglobal\x1b[0m"
    };
    println!("  \x1b[32m✓\x1b[0m {scope} task #{id}: {title}");
    Ok(())
}

fn cmd_list(global: bool, local: bool, all: bool, done: bool) -> Result<()> {
    let config = AppConfig::load_or_default()?;
    let db = Database::open_default()?;
    let (project_folder, in_project) = project_context();

    let scope = if all {
        if local {
            ScopeFilter::LocalAll
        } else if global {
            ScopeFilter::GlobalOnly
        } else {
            ScopeFilter::MergedAll
        }
    } else if global {
        ScopeFilter::GlobalOnly
    } else if local {
        if !in_project {
            bail!("--local requires being inside a git repository");
        }
        ScopeFilter::LocalCurrent
    } else {
        // No flags: use config default_view
        match config.default_view {
            DefaultView::Auto => {
                if in_project {
                    ScopeFilter::LocalCurrent
                } else {
                    ScopeFilter::GlobalOnly
                }
            }
            DefaultView::Merged => {
                if in_project {
                    ScopeFilter::MergedCurrent
                } else {
                    ScopeFilter::GlobalOnly
                }
            }
            DefaultView::Local => {
                if in_project {
                    ScopeFilter::LocalCurrent
                } else {
                    ScopeFilter::GlobalOnly
                }
            }
            DefaultView::Global => ScopeFilter::GlobalOnly,
        }
    };

    let status = if done {
        StatusFilter::CompletedOnly
    } else {
        StatusFilter::PendingOnly
    };

    let tasks = db.list_tasks(ListFilter {
        scope,
        status,
        current_local_folder: project_folder.clone(),
    })?;

    let header = match scope {
        ScopeFilter::LocalCurrent => {
            project_folder
                .as_deref()
                .map(|p| project_name(std::path::Path::new(p)))
                .unwrap_or_else(|| "Tasks".into())
        }
        ScopeFilter::MergedCurrent => {
            project_folder
                .as_deref()
                .map(|p| format!("{} + Global", project_name(std::path::Path::new(p))))
                .unwrap_or_else(|| "Tasks".into())
        }
        ScopeFilter::GlobalOnly => "Global Tasks".into(),
        ScopeFilter::MergedAll => "All Tasks".into(),
        ScopeFilter::LocalAll => "All Project Tasks".into(),
    };

    ui::print_header(&header);
    ui::print_task_table(&tasks, &config);
    println!();
    Ok(())
}

fn cmd_done(id: i64) -> Result<()> {
    Database::open_default()?.mark_done(id)?;
    println!("  \x1b[32m✓\x1b[0m Task #{id} marked as completed.");
    Ok(())
}

fn cmd_edit(id: i64, title: &str) -> Result<()> {
    Database::open_default()?.edit_task(id, title)?;
    println!("  \x1b[32m✓\x1b[0m Task #{id} updated: {title}");
    Ok(())
}

fn cmd_undo() -> Result<()> {
    let task = Database::open_default()?.undo_last_completed()?;
    println!("  \x1b[32m✓\x1b[0m Restored: #{} {}", task.id, task.title);
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

fn cmd_log(all: bool) -> Result<()> {
    let config = AppConfig::load_or_default()?;
    let db = Database::open_default()?;
    let (project_folder, in_project) = project_context();

    let folder_filter = if all || !in_project {
        None
    } else {
        project_folder.as_deref()
    };

    let tasks = db.recent_completed(25, folder_filter)?;

    let header = if !all && in_project {
        let name = project_folder
            .as_deref()
            .map(|p| project_name(std::path::Path::new(p)))
            .unwrap_or_else(|| "project".into());
        format!("Recently Completed · {name}")
    } else {
        "Recently Completed".into()
    };

    ui::print_header(&header);
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
