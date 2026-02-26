use anyhow::{Context, Result};

use crate::config;
use crate::db;

/// Run the uninstall flow: confirm, then delete DB, config, and binary.
pub fn run_uninstall() -> Result<()> {
    let confirm = demand::Confirm::new("Uninstall tody?")
        .description("This will delete the database, config file, and the tody binary itself.")
        .affirmative("Yes, uninstall")
        .negative("Cancel");

    let yes = confirm.run().map_err(|e| {
        if e.kind() == std::io::ErrorKind::Interrupted {
            anyhow::anyhow!("cancelled")
        } else {
            anyhow::anyhow!("prompt error: {e}")
        }
    })?;

    if !yes {
        println!("  Uninstall cancelled.");
        return Ok(());
    }

    // Delete database file
    let db_path = db::default_db_path()?;
    if db_path.exists() {
        std::fs::remove_file(&db_path)
            .with_context(|| format!("failed to delete database at {}", db_path.display()))?;
        println!("  ✓ Deleted database: {}", db_path.display());
        // Also remove the parent dir if empty
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    } else {
        println!("  Database not found (already clean).");
    }

    // Delete config file
    let config_path = config::config_path()?;
    if config_path.exists() {
        std::fs::remove_file(&config_path)
            .with_context(|| format!("failed to delete config at {}", config_path.display()))?;
        println!("  ✓ Deleted config: {}", config_path.display());
        if let Some(parent) = config_path.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    } else {
        println!("  Config not found (already clean).");
    }

    // Delete binary itself
    let exe_path =
        std::env::current_exe().context("failed to determine current executable path")?;

    #[cfg(not(windows))]
    {
        std::fs::remove_file(&exe_path)
            .with_context(|| format!("failed to delete binary at {}", exe_path.display()))?;
        println!("  ✓ Deleted binary: {}", exe_path.display());
    }

    #[cfg(windows)]
    {
        // On Windows, a running executable cannot be deleted directly.
        // Rename it and spawn a deferred delete process.
        let old_path = exe_path.with_extension("old.exe");
        let _ = std::fs::remove_file(&old_path);
        std::fs::rename(&exe_path, &old_path)
            .with_context(|| format!("failed to rename binary at {}", exe_path.display()))?;

        // Spawn a CMD process that waits, then deletes the renamed file
        let _ = std::process::Command::new("cmd")
            .args(["/C", "timeout", "/T", "2", "/NOBREAK", ">NUL", "&", "del"])
            .arg(&old_path)
            .spawn();

        println!(
            "  ✓ Binary will be removed momentarily: {}",
            exe_path.display()
        );
    }

    println!();
    println!("  tody has been uninstalled. Goodbye! 👋");
    Ok(())
}
