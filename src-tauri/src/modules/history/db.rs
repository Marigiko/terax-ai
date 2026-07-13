// Pure SQLite persistence for terminal history. No Tauri command surface here.
// All logic is synchronous (rusqlite is sync); callers hold the Mutex.

use rusqlite::{params, Connection, Result as DbResult};
use std::path::Path;

pub struct FullEntry {
    pub id: i64,
    pub command: String,
    pub timestamp: i64,
    pub exit_code: Option<i32>,
    pub session_id: String,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &Path) -> DbResult<Self> {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }

        let open_and_init = |p: &Path| -> DbResult<Self> {
            let conn = Connection::open(p)?;
            let db = Self { conn };
            db.init()?;
            Ok(db)
        };

        match open_and_init(path) {
            Ok(db) => Ok(db),
            Err(e) => {
                log::warn!("[history] failed to open/init history db: {e}; rotating to .bak");
                let bak = path.with_extension("db.bak");
                let _ = std::fs::rename(path, bak);

                let mut wal = path.as_os_str().to_os_string();
                wal.push("-wal");
                let _ = std::fs::remove_file(wal);

                let mut shm = path.as_os_str().to_os_string();
                shm.push("-shm");
                let _ = std::fs::remove_file(shm);

                open_and_init(path)
            }
        }
    }

    fn init(&self) -> DbResult<()> {
        self.conn.execute_batch(
            "
            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;

            CREATE TABLE IF NOT EXISTS history (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                command    TEXT    NOT NULL,
                timestamp  INTEGER NOT NULL,
                exit_code  INTEGER,
                session_id TEXT    NOT NULL DEFAULT '',
                UNIQUE(command, timestamp)
            );

            CREATE INDEX IF NOT EXISTS idx_history_ts  ON history(timestamp DESC);
            ",
        )
    }

    pub fn insert(
        &self,
        command: &str,
        timestamp: i64,
        exit_code: Option<i32>,
        session_id: &str,
    ) -> DbResult<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO history (command, timestamp, exit_code, session_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![command, timestamp, exit_code, session_id],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_command(&self, id: i64) -> DbResult<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT command FROM history WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete(&self, id: i64) -> DbResult<()> {
        self.conn
            .execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear(&self) -> DbResult<()> {
        self.conn.execute_batch("DELETE FROM history;")?;
        Ok(())
    }

    pub fn load_all(&self) -> DbResult<Vec<(String, i64)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT command, timestamp FROM history ORDER BY timestamp ASC")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    }

    pub fn list(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> DbResult<Vec<FullEntry>> {
        let escaped = query.to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("%{}%", escaped);
        let mut stmt = self.conn.prepare(
            "SELECT id, command, timestamp, exit_code, session_id
             FROM history
             WHERE lower(command) LIKE ?1 ESCAPE '\\'
             ORDER BY timestamp DESC
             LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt.query_map(params![pattern, limit as i64, offset as i64], |row| {
            Ok(FullEntry {
                id: row.get(0)?,
                command: row.get(1)?,
                timestamp: row.get(2)?,
                exit_code: row.get(3)?,
                session_id: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn seed(&self, entries: &[(String, i64)]) -> DbResult<()> {
        let tx = self.conn.unchecked_transaction()?;
        for (cmd, ts) in entries {
            tx.execute(
                "INSERT OR IGNORE INTO history (command, timestamp, session_id)
                 VALUES (?1, ?2, '')",
                params![cmd, ts],
            )?;
        }
        tx.commit()
    }

    pub fn is_empty(&self) -> DbResult<bool> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0))?;
        Ok(count == 0)
    }

    pub fn trim(&self, max: usize) -> DbResult<()> {
        if max == 0 {
            return Ok(());
        }
        self.conn.execute(
            "DELETE FROM history WHERE id IN (
                SELECT id FROM history ORDER BY timestamp ASC, id ASC
                LIMIT MAX(0, (SELECT COUNT(*) FROM history) - ?1)
             )",
            params![max as i64],
        )?;
        Ok(())
    }
}
