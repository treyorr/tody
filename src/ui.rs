use crate::config::AppConfig;
use crate::db::{Task, TaskStatus};
use std::path::Path;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Map a color name from config to an ANSI escape code.
fn ansi_color(name: &str) -> &'static str {
    match name.trim().to_ascii_lowercase().as_str() {
        "black" => "\x1b[30m",
        "red" => "\x1b[31m",
        "green" => "\x1b[32m",
        "yellow" => "\x1b[33m",
        "blue" => "\x1b[34m",
        "magenta" => "\x1b[35m",
        "cyan" => "\x1b[36m",
        "white" => "\x1b[37m",
        "bright_black" | "gray" | "grey" => "\x1b[90m",
        "bright_red" => "\x1b[91m",
        "bright_green" => "\x1b[92m",
        "bright_yellow" => "\x1b[93m",
        "bright_blue" => "\x1b[94m",
        "bright_magenta" => "\x1b[95m",
        "bright_cyan" => "\x1b[96m",
        "bright_white" => "\x1b[97m",
        _ => "\x1b[0m", // fallback to reset/default
    }
}

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const ITALIC: &str = "\x1b[3m";
const STRIKETHROUGH: &str = "\x1b[9m";
const GREEN: &str = "\x1b[32m";
const ACCENT: &str = "\x1b[96m";

const DIAMOND: &str = "◆";
const CIRCLE: &str = "○";
const CHECK: &str = "✓";
const BAR: &str = "▍";
const RULE_CHAR: &str = "─";

// ─── Helpers ─────────────────────────────────────────────────

fn scope_color_code(task: &Task, config: &AppConfig) -> &'static str {
    if task.folder_path.is_some() {
        ansi_color(&config.color_local)
    } else {
        ansi_color(&config.color_global)
    }
}

fn scope_text(task: &Task) -> &'static str {
    if task.folder_path.is_some() {
        "local"
    } else {
        "global"
    }
}

fn short_path(path: &Path) -> String {
    let comps: Vec<&std::ffi::OsStr> = path.components().map(|c| c.as_os_str()).collect();
    if comps.len() <= 2 {
        return path.to_string_lossy().to_string();
    }
    let tail: Vec<&str> = comps[comps.len() - 2..]
        .iter()
        .filter_map(|c| c.to_str())
        .collect();
    format!("…/{}", tail.join("/"))
}

fn relative_time(rfc3339: &str) -> String {
    let Ok(then) = OffsetDateTime::parse(rfc3339, &Rfc3339) else {
        return rfc3339.to_string();
    };
    let diff = OffsetDateTime::now_utc() - then;
    let secs = diff.whole_seconds();
    if secs < 60 {
        return "just now".into();
    }
    let mins = diff.whole_minutes();
    if mins < 60 {
        return format!("{mins}m ago");
    }
    let hours = diff.whole_hours();
    if hours < 24 {
        return format!("{hours}h ago");
    }
    let days = diff.whole_days();
    if days == 1 {
        return "yesterday".into();
    }
    if days < 30 {
        return format!("{days}d ago");
    }
    format!("{}mo ago", days / 30)
}

fn rule(width: usize) -> String {
    RULE_CHAR.repeat(width)
}

fn pad(len: usize, target: usize) -> String {
    if len >= target {
        String::new()
    } else {
        " ".repeat(target - len)
    }
}

// ─── Public API ──────────────────────────────────────────────

/// Section header with accent diamond and thin rule.
pub fn print_header(title: &str) {
    println!();
    println!("  {ACCENT}{DIAMOND}{RESET} {BOLD}{title}{RESET}");
    println!("  {DIM}{}{RESET}", rule(46));
}

/// Styled task table with column-aligned scope, id, and summary footer.
pub fn print_task_table(tasks: &[Task], config: &AppConfig) {
    if tasks.is_empty() {
        println!();
        println!("  {DIM}No tasks to show.{RESET}");
        println!("  {DIM}Run{RESET} tody add \"your task\" {DIM}to get started.{RESET}");
        return;
    }

    let max_w = tasks
        .iter()
        .map(|t| t.title.len())
        .max()
        .unwrap_or(20)
        .clamp(20, 48);

    println!();
    for t in tasks {
        let c = scope_color_code(t, config);
        let scope = scope_text(t);

        let (icon, title_fmt) = match t.status {
            TaskStatus::Pending => (format!("{c}{CIRCLE}{RESET}"), t.title.clone()),
            TaskStatus::Completed => (
                format!("{GREEN}{CHECK}{RESET}"),
                format!("{DIM}{STRIKETHROUGH}{}{RESET}", t.title),
            ),
        };

        let title_pad = pad(t.title.len(), max_w);

        let path_str = t
            .folder_path
            .as_ref()
            .map(|p| format!("  {DIM}{}{RESET}", short_path(p)))
            .unwrap_or_default();

        println!(
            "  {c}{BAR}{RESET} {icon}  {title_fmt}{title_pad}  {c}{ITALIC}{scope:>6}{RESET}  {DIM}#{:<3}{RESET}{path_str}",
            t.id,
        );
    }

    let pending = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Pending)
        .count();
    let done = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Completed)
        .count();

    let mut parts = Vec::new();
    if pending > 0 {
        parts.push(format!("{BOLD}{pending}{RESET} pending"));
    }
    if done > 0 {
        parts.push(format!("{DIM}{done} done{RESET}"));
    }

    println!();
    println!(
        "  {DIM}{}{RESET} {}",
        rule(2),
        parts.join(&format!(" {DIM}·{RESET} "))
    );
}

/// Log-style list of completed tasks with relative timestamps.
pub fn print_log(tasks: &[Task], config: &AppConfig) {
    if tasks.is_empty() {
        println!();
        println!("  {DIM}No completed tasks yet.{RESET}");
        return;
    }

    let max_w = tasks
        .iter()
        .map(|t| t.title.len())
        .max()
        .unwrap_or(20)
        .clamp(20, 48);

    println!();
    for t in tasks {
        let c = scope_color_code(t, config);
        let scope = scope_text(t);
        let when = t
            .completed_at
            .as_deref()
            .map(relative_time)
            .unwrap_or_else(|| "—".into());

        let title_pad = pad(t.title.len(), max_w);

        println!(
            "  {c}{BAR}{RESET} {GREEN}{CHECK}{RESET}  {DIM}{}{RESET}{title_pad}  {c}{ITALIC}{scope:>6}{RESET}  {DIM}#{:<3}{RESET}  {DIM}{when}{RESET}",
            t.title, t.id,
        );
    }

    println!();
    println!("  {DIM}{} {} completed{RESET}", rule(2), tasks.len());
}

/// Format a task for the interactive `demand` select list.
pub fn format_task_option(task: &Task, config: &AppConfig) -> String {
    let c = scope_color_code(task, config);
    let scope = scope_text(task);
    let path_str = task
        .folder_path
        .as_ref()
        .map(|p| format!(" {DIM}· {}{RESET}", short_path(p)))
        .unwrap_or_default();

    format!(
        "{c}{BAR}{RESET} {title}  {c}{ITALIC}{scope}{RESET}{path_str}",
        title = task.title
    )
}

/// Inline scope label for success/info messages.
pub fn scope_label(task: &Task, config: &AppConfig) -> String {
    let c = scope_color_code(task, config);
    let label = if task.folder_path.is_some() {
        "Local"
    } else {
        "Global"
    };
    format!("{c}{label}{RESET}")
}
