use anyhow::Result;

/// Run the update flow using `self_update` to check GitHub for a newer release
/// and perform an in-place binary replacement.
///
/// Versions follow CalVer (`YYYY.MM.COUNTER`), e.g. `2026.2.1`.
///
/// Release assets should follow the naming convention:
///   `tody-v{version}-{target}.tar.gz`  (e.g. `tody-v2026.2.1-aarch64-apple-darwin.tar.gz`)
pub fn run_update(current_version: &str) -> Result<()> {
    println!("  Checking for updates…");

    let status = self_update::backends::github::Update::configure()
        .repo_owner("treyorr")
        .repo_name("tody")
        .bin_name("tody")
        .current_version(current_version)
        .show_download_progress(true)
        .no_confirm(true)
        .build()?
        .update()?;

    if status.updated() {
        println!("  ✓ Updated to v{}!", status.version());
    } else {
        println!("  ✓ Already up to date (v{current_version}).");
    }

    Ok(())
}
