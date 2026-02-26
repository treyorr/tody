use anyhow::{Context, Result, anyhow, bail};
use dirs::data_local_dir;
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};
use std::process::Command;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const APP_DIR: &str = "tody";
const DB_FILENAME: &str = "tody.db";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Completed,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = anyhow::Error;

    fn from_str(raw: &str) -> Result<Self> {
        match raw {
            "pending" => Ok(Self::Pending),
            "completed" => Ok(Self::Completed),
            _ => bail!("invalid status value in database: {raw}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub status: TaskStatus,
    pub folder_path: Option<PathBuf>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeFilter {
    Merged,
    GlobalOnly,
    LocalOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListFilter {
    pub scope: ScopeFilter,
    pub include_completed: bool,
}

impl Default for ListFilter {
    fn default() -> Self {
        Self {
            scope: ScopeFilter::Merged,
            include_completed: false,
        }
    }
}

#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open_default() -> Result<Self> {
        let path = default_db_path()?;
        Self::open_at(path)
    }

    pub fn open_at(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create db directory {}", parent.display()))?;
        }

        let conn = Connection::open(path)
            .with_context(|| format!("failed to open SQLite db at {}", path.display()))?;
        let db = Self { conn };
        db.ensure_schema()?;
        Ok(db)
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn ensure_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
              id INTEGER PRIMARY KEY,
              title TEXT NOT NULL,
              status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'completed')),
              folder_path TEXT,
              created_at DATETIME NOT NULL,
              completed_at DATETIME
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_status_created ON tasks(status, created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_tasks_folder_path ON tasks(folder_path);
            CREATE INDEX IF NOT EXISTS idx_tasks_completed_at ON tasks(completed_at DESC);
            "#,
        )?;
        Ok(())
    }

    pub fn add_task(&self, title: &str, folder_path: Option<&Path>) -> Result<i64> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            bail!("task title cannot be empty");
        }

        let created_at = now_utc_rfc3339()?;
        let folder_path = folder_path.map(normalize_folder_path).transpose()?;

        self.conn.execute(
            r#"
            INSERT INTO tasks(title, status, folder_path, created_at, completed_at)
            VALUES (?1, 'pending', ?2, ?3, NULL)
            "#,
            params![trimmed, folder_path, created_at],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn mark_done(&self, id: i64) -> Result<()> {
        let completed_at = now_utc_rfc3339()?;
        let changed = self.conn.execute(
            r#"
            UPDATE tasks
            SET status = 'completed', completed_at = ?2
            WHERE id = ?1 AND status != 'completed'
            "#,
            params![id, completed_at],
        )?;

        if changed == 0 {
            bail!("task {id} was not updated (not found or already completed)");
        }

        Ok(())
    }

    pub fn remove_task(&self, id: i64) -> Result<()> {
        let changed = self
            .conn
            .execute("DELETE FROM tasks WHERE id = ?1", params![id])?;

        if changed == 0 {
            bail!("task {id} does not exist");
        }

        Ok(())
    }

    pub fn list_tasks(&self, filter: ListFilter) -> Result<Vec<Task>> {
        let mut sql = String::from(
            "SELECT id, title, status, folder_path, created_at, completed_at FROM tasks WHERE 1 = 1",
        );

        if !filter.include_completed {
            sql.push_str(" AND status = 'pending'");
        }

        match filter.scope {
            ScopeFilter::Merged => {}
            ScopeFilter::GlobalOnly => sql.push_str(" AND folder_path IS NULL"),
            ScopeFilter::LocalOnly => sql.push_str(" AND folder_path IS NOT NULL"),
        }

        sql.push_str(" ORDER BY created_at DESC, id DESC");

        let mut stmt = self.conn.prepare(&sql)?;
        let mapped = stmt.query_map([], |row| {
            let status_raw: String = row.get(2)?;
            let status = match status_raw.as_str() {
                "pending" => TaskStatus::Pending,
                "completed" => TaskStatus::Completed,
                other => {
                    return Err(rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("invalid status: {other}"),
                        )),
                    ));
                }
            };

            let folder_path: Option<String> = row.get(3)?;
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                status,
                folder_path: folder_path.map(PathBuf::from),
                created_at: row.get(4)?,
                completed_at: row.get(5)?,
            })
        })?;

        mapped
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn recent_completed(&self, limit: usize) -> Result<Vec<Task>> {
        let limit_i64 =
            i64::try_from(limit.max(1)).map_err(|_| anyhow!("limit value is too large"))?;
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, title, status, folder_path, created_at, completed_at
            FROM tasks
            WHERE status = 'completed'
            ORDER BY completed_at DESC, id DESC
            LIMIT ?1
            "#,
        )?;

        let mapped = stmt.query_map(params![limit_i64], |row| {
            let folder_path: Option<String> = row.get(3)?;
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                status: TaskStatus::Completed,
                folder_path: folder_path.map(PathBuf::from),
                created_at: row.get(4)?,
                completed_at: row.get(5)?,
            })
        })?;

        mapped
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn orphaned_folder_paths(&self) -> Result<Vec<PathBuf>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT DISTINCT folder_path
            FROM tasks
            WHERE folder_path IS NOT NULL
            ORDER BY folder_path
            "#,
        )?;

        let folders = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut orphaned = Vec::new();

        for folder in folders {
            let path = PathBuf::from(folder?);
            if !path.exists() {
                orphaned.push(path);
            }
        }

        Ok(orphaned)
    }

    pub fn delete_tasks_for_folder_paths(&mut self, folder_paths: &[PathBuf]) -> Result<usize> {
        if folder_paths.is_empty() {
            return Ok(0);
        }

        let tx = self.conn.transaction()?;
        let mut total_deleted = 0usize;

        for path in folder_paths {
            let as_text = path.to_string_lossy().to_string();
            total_deleted +=
                tx.execute("DELETE FROM tasks WHERE folder_path = ?1", params![as_text])?;
        }

        tx.commit()?;
        Ok(total_deleted)
    }
}

pub fn default_db_path() -> Result<PathBuf> {
    let base = data_local_dir().ok_or_else(|| anyhow!("unable to resolve local data directory"))?;
    Ok(base.join(APP_DIR).join(DB_FILENAME))
}

pub fn resolve_local_folder_path() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("failed to get current working directory")?;

    let git_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&current_dir)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            let raw = String::from_utf8(output.stdout).ok()?;
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(PathBuf::from(trimmed))
            }
        });

    let candidate = git_root.unwrap_or(current_dir);
    normalize_path(candidate)
}

fn normalize_folder_path(path: &Path) -> Result<String> {
    normalize_path(path.to_path_buf()).map(|p| p.to_string_lossy().to_string())
}

fn normalize_path(path: PathBuf) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .context("failed to resolve current working directory")?
            .join(path)
    };

    match absolute.canonicalize() {
        Ok(canonical) => Ok(canonical),
        Err(_) => Ok(absolute),
    }
}

fn now_utc_rfc3339() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("failed to format UTC timestamp")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn creates_schema_and_round_trips_tasks() -> Result<()> {
        let tmp = tempdir()?;
        let db = Database::open_at(tmp.path().join("tody.db"))?;

        let id = db.add_task("write tests", None)?;
        db.mark_done(id)?;

        let completed = db.recent_completed(10)?;
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].title, "write tests");
        assert_eq!(completed[0].status, TaskStatus::Completed);
        Ok(())
    }

    #[test]
    fn add_and_remove_task() -> Result<()> {
        let tmp = tempdir()?;
        let db = Database::open_at(tmp.path().join("tody.db"))?;

        let id = db.add_task("temporary task", None)?;
        db.remove_task(id)?;

        let tasks = db.list_tasks(ListFilter::default())?;
        assert!(tasks.iter().all(|t| t.id != id));
        Ok(())
    }

    #[test]
    fn mark_done_twice_errors() -> Result<()> {
        let tmp = tempdir()?;
        let db = Database::open_at(tmp.path().join("tody.db"))?;

        let id = db.add_task("do once", None)?;
        db.mark_done(id)?;

        let result = db.mark_done(id);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn remove_nonexistent_errors() -> Result<()> {
        let tmp = tempdir()?;
        let db = Database::open_at(tmp.path().join("tody.db"))?;

        let result = db.remove_task(99999);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn empty_title_rejected() -> Result<()> {
        let tmp = tempdir()?;
        let db = Database::open_at(tmp.path().join("tody.db"))?;

        let result = db.add_task("   ", None);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn detects_orphaned_folder_paths() -> Result<()> {
        let tmp = tempdir()?;
        let db = Database::open_at(tmp.path().join("tody.db"))?;

        let fake_dir = tmp.path().join("nonexistent_project");
        db.add_task("orphan task", Some(fake_dir.as_path()))?;
        db.add_task("global task", None)?;

        let orphans = db.orphaned_folder_paths()?;
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0], fake_dir);
        Ok(())
    }

    #[test]
    fn list_filter_scopes_work() -> Result<()> {
        let tmp = tempdir()?;
        let db = Database::open_at(tmp.path().join("tody.db"))?;

        db.add_task("global one", None)?;
        // Use tmp as a local path (it exists)
        db.add_task("local one", Some(tmp.path()))?;

        let global_only = db.list_tasks(ListFilter {
            scope: ScopeFilter::GlobalOnly,
            include_completed: false,
        })?;
        assert!(global_only.iter().all(|t| t.folder_path.is_none()));
        assert_eq!(global_only.len(), 1);

        let local_only = db.list_tasks(ListFilter {
            scope: ScopeFilter::LocalOnly,
            include_completed: false,
        })?;
        assert!(local_only.iter().all(|t| t.folder_path.is_some()));
        assert_eq!(local_only.len(), 1);

        let merged = db.list_tasks(ListFilter::default())?;
        assert_eq!(merged.len(), 2);

        Ok(())
    }
}
