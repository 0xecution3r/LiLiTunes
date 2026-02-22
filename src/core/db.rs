use crate::core::model::Track;
use rusqlite::{Connection, Result};

#[derive(Clone)]
pub struct Library {
    db_path: String,
}

impl Library {
    pub fn new() -> Result<Self> {
        let db_path = "library.db".to_string();
        let lib = Self { db_path };
        lib.init()?;
        Ok(lib)
    }

    fn conn(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
    }

    fn init(&self) -> Result<()> {
        let conn = self.conn()?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS tracks (
                id      INTEGER PRIMARY KEY,
                path    TEXT NOT NULL UNIQUE,
                title   TEXT NOT NULL,
                artist  TEXT NOT NULL,
                album   TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn add_track_stub(&self, path: &str) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR IGNORE INTO tracks (path, title, artist, album) VALUES (?1, ?2, ?3, ?4)",
            (path, "Unknown Title", "Unknown Artist", "Unknown Album"),
        )?;
        Ok(())
    }

    pub fn all_tracks(&self) -> Result<Vec<Track>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, path, title, artist, album FROM tracks ORDER BY artist, album, title",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Track {
                id: row.get(0)?,
                path: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                album: row.get(4)?,
            })
        })?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn upsert_track(
        &self,
        path: &str,
        artist: &str,
        title: &str,
        album: &str,
    ) -> rusqlite::Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO tracks (path, artist, title, album)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(path) DO UPDATE SET
           artist=excluded.artist,
           title=excluded.title,
           album=excluded.album",
            rusqlite::params![path, artist, title, album],
        )?;
        Ok(())
    }
}
