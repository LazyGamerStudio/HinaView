use anyhow::Result;
use crossbeam::channel::{Sender, unbounded};
use parking_lot::Mutex;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

use crate::settings::model::ConfigStorageLocation;

enum DbMessage {
    SaveHistory {
        path_hash: String,
        abs_path: String,
        page_index: i64,
    },
    SaveBookmark(crate::bookmark::model::BookmarkEntry),
    DeleteBookmark(u64),
    Shutdown(Sender<()>),
}

pub struct DatabaseService {
    conn: Arc<Mutex<Connection>>,
    tx: Sender<DbMessage>,
}

impl DatabaseService {
    pub fn new_with_location(location: ConfigStorageLocation) -> Result<Self> {
        let db_path = Self::db_path_for_location(location)?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!("Opening database at {:?}", db_path);
        let conn = Connection::open(&db_path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        let conn = Arc::new(Mutex::new(conn));

        {
            let c = conn.lock();
            Self::init_schema(&c)?;
        }

        let (tx, rx) = unbounded::<DbMessage>();

        let worker_conn = Connection::open(db_path)?;
        std::thread::spawn(move || {
            let mut conn = worker_conn;
            while let Ok(msg) = rx.recv() {
                match msg {
                    DbMessage::Shutdown(ack) => {
                        let _ = ack.send(());
                        break;
                    }
                    _ => {
                        if let Err(e) = Self::handle_message(&mut conn, msg) {
                            error!("[Database][Worker] Error: {}", e);
                        }
                    }
                }
            }
        });

        Ok(Self { conn, tx })
    }

    pub fn close(&self) {
        let (ack_tx, ack_rx) = unbounded();
        let _ = self.tx.send(DbMessage::Shutdown(ack_tx));
        let _ = ack_rx.recv_timeout(std::time::Duration::from_secs(2));
        info!("Database service closed gracefully.");
    }

    fn handle_message(conn: &mut Connection, msg: DbMessage) -> Result<()> {
        match msg {
            DbMessage::SaveHistory {
                path_hash,
                abs_path,
                page_index,
            } => {
                conn.execute(
                    "INSERT INTO history (path_hash, abs_path, page_index, last_viewed_at)
                     VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
                     ON CONFLICT(path_hash) DO UPDATE SET
                        page_index = excluded.page_index,
                        last_viewed_at = excluded.last_viewed_at",
                    params![path_hash, abs_path, page_index],
                )?;
            }
            DbMessage::SaveBookmark(entry) => {
                let source_str = match entry.source {
                    crate::bookmark::model::BookmarkSource::AutoRecent => "AutoRecent",
                    crate::bookmark::model::BookmarkSource::Manual => "Manual",
                };

                conn.execute(
                    "INSERT INTO bookmarks (source, archive_name, file_name, path, page_index, page_name, saved_at_ms)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(path, source) DO UPDATE SET
                        page_index = excluded.page_index,
                        saved_at_ms = excluded.saved_at_ms",
                    params![source_str, entry.archive_name, entry.file_name, entry.path.to_string_lossy(), entry.page_index as i64, entry.page_name, entry.saved_at_ms as i64],
                )?;

                // Enforce 5-entry limit for AutoRecent
                if entry.source == crate::bookmark::model::BookmarkSource::AutoRecent {
                    conn.execute(
                        "DELETE FROM bookmarks WHERE source = 'AutoRecent' AND id NOT IN (
                            SELECT id FROM bookmarks WHERE source = 'AutoRecent'
                            ORDER BY saved_at_ms DESC LIMIT 5
                        )",
                        [],
                    )?;
                }
            }
            DbMessage::DeleteBookmark(id) => {
                let _ = conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id as i64]);
            }
            DbMessage::Shutdown(_) => {}
        }
        Ok(())
    }

    fn db_path_for_location(location: ConfigStorageLocation) -> Result<PathBuf> {
        match location {
            ConfigStorageLocation::AppDir => {
                let exe_path = std::env::current_exe()
                    .map_err(|e| anyhow::anyhow!("Failed to get executable path: {}", e))?;
                let app_dir = exe_path
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("Failed to get executable directory"))?
                    .to_path_buf();
                Ok(app_dir.join("config").join("hinaview.db"))
            }
            ConfigStorageLocation::SystemConfig => {
                let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("No config dir"))?;
                Ok(base.join("HinaView").join("hinaview.db"))
            }
        }
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                path_hash TEXT PRIMARY KEY,
                abs_path TEXT NOT NULL,
                page_index INTEGER NOT NULL,
                last_viewed_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source TEXT NOT NULL,
                archive_name TEXT NOT NULL,
                file_name TEXT NOT NULL,
                path TEXT NOT NULL,
                page_index INTEGER NOT NULL,
                page_name TEXT NOT NULL,
                saved_at_ms INTEGER NOT NULL,
                UNIQUE(path, source) -- Ensure one entry per path per source
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bookmarks_path ON bookmarks(path)",
            [],
        )?;
        Ok(())
    }

    pub fn save_resume_position(&self, abs_path: &str, page_index: usize) -> Result<()> {
        let _ = self.tx.send(DbMessage::SaveHistory {
            path_hash: self.hash_path(abs_path),
            abs_path: abs_path.to_string(),
            page_index: page_index as i64,
        });
        Ok(())
    }

    pub fn save_bookmark(&self, entry: &crate::bookmark::model::BookmarkEntry) -> Result<()> {
        let _ = self.tx.send(DbMessage::SaveBookmark(entry.clone()));
        Ok(())
    }

    pub fn delete_bookmark(&self, id: u64) -> Result<()> {
        let _ = self.tx.send(DbMessage::DeleteBookmark(id));
        Ok(())
    }

    pub fn load_resume_position(&self, abs_path: &str) -> Result<Option<usize>> {
        let path_hash = self.hash_path(abs_path);
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT page_index FROM history WHERE path_hash = ?1")?;
        let mut rows = stmt.query(params![path_hash])?;
        if let Some(row) = rows.next()? {
            let idx: i64 = row.get(0)?;
            Ok(Some(idx as usize))
        } else {
            Ok(None)
        }
    }

    pub fn load_bookmarks(&self) -> Result<Vec<crate::bookmark::model::BookmarkEntry>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, source, archive_name, file_name, path, page_index, page_name, saved_at_ms
             FROM bookmarks ORDER BY saved_at_ms DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let source_str: String = row.get(1)?;
            let source = if source_str == "Manual" {
                crate::bookmark::model::BookmarkSource::Manual
            } else {
                crate::bookmark::model::BookmarkSource::AutoRecent
            };
            Ok(crate::bookmark::model::BookmarkEntry {
                id: row.get::<_, i64>(0)? as u64,
                source,
                archive_name: row.get(2)?,
                file_name: row.get(3)?,
                path: PathBuf::from(row.get::<_, String>(4)?),
                page_index: row.get::<_, i64>(5)? as usize,
                page_name: row.get(6)?,
                saved_at_ms: row.get::<_, i64>(7)? as u64,
            })
        })?;
        let mut bookmarks = Vec::new();
        for b in rows {
            bookmarks.push(b?);
        }
        Ok(bookmarks)
    }

    fn hash_path(&self, path: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
