use crate::{
    catalog::Track,
    queue::{Queue, Repeat},
};
use rusqlite::{Connection, params};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Debug)]
pub struct Prepared {
    pub path: String,
    pub key: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub seconds: f64,
}
#[derive(Clone, Default)]
pub struct Snapshot {
    pub tracks: Arc<Vec<Track>>,
    pub likes: BTreeSet<usize>,
    pub playlists: Vec<(i64, String)>,
    pub members: Vec<(i64, usize)>,
    pub dark: bool,
    pub close_to_tray: bool,
}
pub struct Library {
    pub connection: Connection,
}
impl Library {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        let version: i64 = connection.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version > 2 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        if version == 1 {
            connection.execute_batch("BEGIN IMMEDIATE; ALTER TABLE tracks ADD COLUMN video_id TEXT; PRAGMA user_version=2; COMMIT;")?;
        }
        connection.busy_timeout(std::time::Duration::from_secs(2))?;
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;
            CREATE TABLE IF NOT EXISTS tracks(id INTEGER PRIMARY KEY, path TEXT NOT NULL, path_key TEXT NOT NULL UNIQUE, title TEXT NOT NULL, artist TEXT NOT NULL, album TEXT NOT NULL, seconds REAL NOT NULL, video_id TEXT);
            CREATE UNIQUE INDEX IF NOT EXISTS online_identity ON tracks(video_id) WHERE video_id IS NOT NULL;
            CREATE TABLE IF NOT EXISTS likes(track_id INTEGER PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE);
            CREATE TABLE IF NOT EXISTS playlists(id INTEGER PRIMARY KEY, name TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS playlist_tracks(playlist_id INTEGER REFERENCES playlists(id) ON DELETE CASCADE, track_id INTEGER REFERENCES tracks(id) ON DELETE CASCADE, ordinal INTEGER NOT NULL, PRIMARY KEY(playlist_id,track_id));
            CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS queue(ordinal INTEGER PRIMARY KEY, track_id INTEGER NOT NULL REFERENCES tracks(id));
            CREATE TABLE IF NOT EXISTS canonical_queue(ordinal INTEGER PRIMARY KEY, track_id INTEGER NOT NULL REFERENCES tracks(id));
            PRAGMA user_version=2;")?;
        Ok(Self { connection })
    }
    pub fn import(&mut self, items: &[Prepared]) -> rusqlite::Result<usize> {
        let tx = self.connection.transaction()?;
        let mut added = 0;
        for item in items {
            added += tx.execute("INSERT OR IGNORE INTO tracks(path,path_key,title,artist,album,seconds) VALUES(?1,?2,?3,?4,?5,?6)",params![item.path,item.key,item.title,item.artist,item.album,item.seconds])?;
        }
        tx.commit()?;
        Ok(added)
    }
    pub fn save_online(&mut self, songs: &[crate::online::Song]) -> rusqlite::Result<Vec<usize>> {
        let tx = self.connection.transaction()?;
        let mut ids = Vec::new();
        for s in songs {
            if !crate::online::valid_id(&s.video_id) {
                return Err(rusqlite::Error::InvalidQuery);
            }
            let key = format!("youtube:{}", s.video_id);
            tx.execute("INSERT OR IGNORE INTO tracks(path,path_key,title,artist,album,seconds,video_id) VALUES(?1,?1,?2,?3,?4,?5,?6)",params![key,s.title,s.artist,s.album,s.seconds,s.video_id])?;
            ids.push(tx.query_row(
                "SELECT id FROM tracks WHERE video_id=?1",
                [&s.video_id],
                |r| r.get::<_, i64>(0),
            )? as usize);
        }
        tx.commit()?;
        Ok(ids)
    }
    pub fn snapshot(&self) -> rusqlite::Result<Snapshot> {
        let tracks = self
            .connection
            .prepare("SELECT id,path,title,artist,album,seconds,video_id FROM tracks ORDER BY id")?
            .query_map([], |r| {
                if let Some(video_id) = r.get::<_, Option<String>>(6)? {
                    return Ok(Track::online(
                        r.get::<_, i64>(0)? as usize,
                        &crate::online::Song {
                            video_id,
                            title: r.get(2)?,
                            artist: r.get(3)?,
                            album: r.get(4)?,
                            seconds: r.get(5)?,
                        },
                    ));
                }
                Ok(Track::local(
                    r.get::<_, i64>(0)? as usize,
                    PathBuf::from(r.get::<_, String>(1)?),
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let likes = self
            .connection
            .prepare("SELECT track_id FROM likes")?
            .query_map([], |r| Ok(r.get::<_, i64>(0)? as usize))?
            .collect::<rusqlite::Result<BTreeSet<_>>>()?;
        let playlists = self
            .connection
            .prepare("SELECT id,name FROM playlists ORDER BY id")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let members = self
            .connection
            .prepare(
                "SELECT playlist_id,track_id FROM playlist_tracks ORDER BY playlist_id,ordinal",
            )?
            .query_map([], |r| Ok((r.get(0)?, r.get::<_, i64>(1)? as usize)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(Snapshot {
            tracks: Arc::new(tracks),
            likes,
            playlists,
            members,
            dark: self.setting("dark").as_deref() != Some("false"),
            close_to_tray: self.setting("close_to_tray").as_deref() == Some("true"),
        })
    }
    pub fn setting(&self, key: &str) -> Option<String> {
        self.connection
            .query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .ok()
    }
    pub fn set(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.connection.execute("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[key,value])?;
        Ok(())
    }
    pub fn toggle_like(&self, id: usize) -> rusqlite::Result<()> {
        if self
            .connection
            .execute("DELETE FROM likes WHERE track_id=?1", [id as i64])?
            == 0
        {
            self.connection
                .execute("INSERT INTO likes VALUES(?1)", [id as i64])?;
        }
        Ok(())
    }
    pub fn create_playlist(&self, name: &str) -> rusqlite::Result<()> {
        let name: String = name.trim().chars().take(120).collect();
        if !name.is_empty() {
            self.connection
                .execute("INSERT INTO playlists(name) VALUES(?1)", [name])?;
        }
        Ok(())
    }
    pub fn add_to_playlist(&self, playlist: i64, track: usize) -> rusqlite::Result<()> {
        self.connection.execute("INSERT OR IGNORE INTO playlist_tracks(playlist_id,track_id,ordinal) SELECT ?1,?2,COALESCE(MAX(ordinal)+1,0) FROM playlist_tracks WHERE playlist_id=?1",params![playlist,track as i64])?;
        Ok(())
    }
    pub fn save_queue(&mut self, q: &Queue, volume: f64) -> rusqlite::Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute("DELETE FROM queue", [])?;
        tx.execute("DELETE FROM canonical_queue", [])?;
        for (n, id) in q.order.iter().enumerate() {
            tx.execute(
                "INSERT INTO queue VALUES(?1,?2)",
                params![n as i64, *id as i64],
            )?;
        }
        for (n, id) in q.canonical.iter().enumerate() {
            tx.execute(
                "INSERT INTO canonical_queue VALUES(?1,?2)",
                params![n as i64, *id as i64],
            )?;
        }
        for (key, value) in [
            ("queue_cursor", q.cursor.to_string()),
            ("queue_position", q.position.to_string()),
            ("shuffle", q.shuffle.to_string()),
            ("repeat", q.repeat.number().to_string()),
            ("volume", volume.to_string()),
        ] {
            tx.execute("INSERT INTO settings VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,value])?;
        }
        tx.commit()
    }
    pub fn save_progress(&mut self, q: &Queue, volume: f64) -> rusqlite::Result<()> {
        let tx = self.connection.transaction()?;
        for (key, value) in [
            ("queue_cursor", q.cursor.to_string()),
            ("queue_position", q.position.to_string()),
            ("shuffle", q.shuffle.to_string()),
            ("repeat", q.repeat.number().to_string()),
            ("volume", volume.to_string()),
        ] {
            tx.execute("INSERT INTO settings VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",params![key,value])?;
        }
        tx.commit()
    }
    pub fn restore_queue(&self) -> rusqlite::Result<Queue> {
        let read = |table: &str| -> rusqlite::Result<Vec<usize>> {
            self.connection
                .prepare(&format!("SELECT track_id FROM {table} ORDER BY ordinal"))?
                .query_map([], |r| Ok(r.get::<_, i64>(0)? as usize))?
                .collect()
        };
        let order = read("queue")?;
        let canonical = read("canonical_queue")?;
        let cursor = self
            .setting("queue_cursor")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0usize)
            .min(order.len().saturating_sub(1));
        let position: f64 = self
            .setting("queue_position")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        Ok(Queue {
            order,
            canonical,
            cursor,
            position: if position.is_finite() {
                position.max(0.0)
            } else {
                0.0
            },
            shuffle: self.setting("shuffle").as_deref() == Some("true"),
            repeat: Repeat::from_number(
                self.setting("repeat")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
            ),
        })
    }
}

pub fn scan(path: &Path) -> Result<Prepared, String> {
    lofty::config::apply_global_options(
        lofty::config::GlobalOptions::new()
            .allocation_limit(4 * 1024 * 1024)
            .preserve_format_specific_items(false),
    );
    use lofty::{
        file::{AudioFile, TaggedFileExt},
        tag::Accessor,
    };
    let path = path.canonicalize().map_err(|e| e.to_string())?;
    let text = path
        .to_str()
        .ok_or("This file path cannot be represented as Unicode")?
        .to_owned();
    let options = lofty::config::ParseOptions::new().read_cover_art(false);
    let audio = lofty::probe::Probe::open(&path)
        .map_err(|e| e.to_string())?
        .options(options)
        .read()
        .map_err(|e| e.to_string())?;
    let tag = audio.primary_tag().or_else(|| audio.first_tag());
    let title = tag
        .and_then(|t| t.title())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.into_owned())
        .unwrap_or_else(|| {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        });
    let artist = tag
        .and_then(|t| t.artist())
        .map(|s| s.into_owned())
        .unwrap_or_else(|| "Unknown artist".into());
    let album = tag
        .and_then(|t| t.album())
        .map(|s| s.into_owned())
        .unwrap_or_else(|| "Local music".into());
    #[cfg(windows)]
    let key = text.to_lowercase();
    #[cfg(not(windows))]
    let key = text.clone();
    Ok(Prepared {
        path: text,
        key,
        title: title
            .chars()
            .filter(|c| !c.is_control())
            .take(512)
            .collect(),
        artist: artist
            .chars()
            .filter(|c| !c.is_control())
            .take(512)
            .collect(),
        album: album
            .chars()
            .filter(|c| !c.is_control())
            .take(512)
            .collect(),
        seconds: audio.properties().duration().as_secs_f64(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(n: usize) -> Prepared {
        Prepared {
            path: format!("C:/Music/{n}.flac"),
            key: format!("c:/music/{n}.flac"),
            title: format!("Track {n}"),
            artist: "শিল্পী".into(),
            album: "Album".into(),
            seconds: 125.0,
        }
    }
    #[test]
    fn dedup_likes_playlists_and_queue_survive_reopening() {
        let p = std::env::temp_dir().join(format!("museflow-store-{}.sqlite", std::process::id()));
        {
            let mut db = Library::open(&p).unwrap();
            assert_eq!(db.import(&[item(1), item(2)]).unwrap(), 2);
            assert_eq!(db.import(&[item(1)]).unwrap(), 0);
            let s = db.snapshot().unwrap();
            let id = s.tracks[0].id;
            db.toggle_like(id).unwrap();
            db.create_playlist("  Night  ").unwrap();
            let playlist = db.snapshot().unwrap().playlists[0].0;
            db.add_to_playlist(playlist, id).unwrap();
            db.add_to_playlist(playlist, id).unwrap();
            let mut q = Queue::default();
            q.replace(s.tracks.iter().map(|t| t.id).collect(), id);
            q.position = 32.5;
            q.repeat = Repeat::All;
            db.save_queue(&q, 31.0).unwrap();
            db.set("dark", "false").unwrap();
        }
        {
            let db = Library::open(&p).unwrap();
            let s = db.snapshot().unwrap();
            assert_eq!(s.tracks.len(), 2);
            assert_eq!(s.likes.len(), 1);
            assert_eq!(s.members.len(), 1);
            assert!(!s.dark);
            assert_eq!(s.tracks[0].artist, "শিল্পী");
            let q = db.restore_queue().unwrap();
            assert_eq!(q.position, 32.5);
            assert_eq!(q.repeat, Repeat::All);
            assert_eq!(q.order.len(), 2);
        }
        std::fs::remove_file(p).unwrap();
    }
    #[test]
    fn invalid_playlist_membership_does_not_insert_or_hide_errors() {
        let db = Library::open(Path::new(":memory:")).unwrap();
        assert!(db.add_to_playlist(77, 99).is_err());
        assert!(db.snapshot().unwrap().members.is_empty());
    }
    #[test]
    fn progress_checkpoint_never_rewrites_queue_and_failed_replacement_rolls_back() {
        let mut db = Library::open(Path::new(":memory:")).unwrap();
        db.import(&[item(1), item(2)]).unwrap();
        let ids = db
            .snapshot()
            .unwrap()
            .tracks
            .iter()
            .map(|t| t.id)
            .collect::<Vec<_>>();
        let mut q = Queue::default();
        q.replace(ids.clone(), ids[0]);
        db.save_queue(&q, 20.0).unwrap();
        db.connection.execute_batch("CREATE TRIGGER reject_queue_delete BEFORE DELETE ON queue BEGIN SELECT RAISE(ABORT,'test fault'); END;").unwrap();
        q.position = 4.5;
        db.save_progress(&q, 17.0).unwrap();
        assert_eq!(db.restore_queue().unwrap().position, 4.5);
        assert!(db.save_queue(&q, 90.0).is_err());
        let restored = db.restore_queue().unwrap();
        assert_eq!(restored.order, ids);
        assert_eq!(db.setting("volume").unwrap(), "17");
    }
    #[test]
    fn import_failure_rolls_back_the_entire_batch() {
        let mut db = Library::open(Path::new(":memory:")).unwrap();
        db.connection.execute_batch("CREATE TRIGGER reject_second BEFORE INSERT ON tracks WHEN NEW.title='Track 2' BEGIN SELECT RAISE(ABORT,'test fault'); END;").unwrap();
        assert!(db.import(&[item(1), item(2)]).is_err());
        assert!(db.snapshot().unwrap().tracks.is_empty());
    }
    #[test]
    fn future_schema_is_rejected_without_downgrading() {
        let path =
            std::env::temp_dir().join(format!("museflow-future-{}.sqlite", std::process::id()));
        {
            let connection = Connection::open(&path).unwrap();
            connection.pragma_update(None, "user_version", 99).unwrap();
        }
        assert!(Library::open(&path).is_err());
        {
            let connection = Connection::open(&path).unwrap();
            assert_eq!(
                connection
                    .pragma_query_value(None, "user_version", |r| r.get::<_, i32>(0))
                    .unwrap(),
                99
            );
        }
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn migrates_v1_without_losing_local_library_likes_or_queue() {
        let path =
            std::env::temp_dir().join(format!("museflow-migrate-{}.sqlite", std::process::id()));
        {
            let mut db = Library::open(&path).unwrap();
            db.import(&[item(1)]).unwrap();
            let id = db.snapshot().unwrap().tracks[0].id;
            db.toggle_like(id).unwrap();
            let mut q = Queue::default();
            q.replace(vec![id], id);
            q.position = 3.0;
            db.save_queue(&q, 25.0).unwrap();
            db.connection.execute_batch("DROP INDEX online_identity; ALTER TABLE tracks DROP COLUMN video_id; PRAGMA user_version=1;").unwrap();
        }
        {
            let db = Library::open(&path).unwrap();
            let s = db.snapshot().unwrap();
            assert_eq!(s.tracks.len(), 1);
            assert!(s.likes.contains(&s.tracks[0].id));
            assert!(s.tracks[0].path.is_some());
            assert!(s.tracks[0].video_id.is_none());
            assert_eq!(db.restore_queue().unwrap().position, 3.0);
            assert_eq!(
                db.connection
                    .pragma_query_value(None, "user_version", |r| r.get::<_, i32>(0))
                    .unwrap(),
                2
            );
        }
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn online_metadata_deduplicates_without_persisting_playable_urls() {
        let mut db = Library::open(Path::new(":memory:")).unwrap();
        let s = crate::online::Song {
            video_id: "abcdefghijk".into(),
            title: "A song".into(),
            artist: "Singer".into(),
            album: "Album".into(),
            seconds: 192.0,
        };
        let a = db.save_online(std::slice::from_ref(&s)).unwrap();
        let b = db.save_online(std::slice::from_ref(&s)).unwrap();
        assert_eq!(a, b);
        let rows = db.snapshot().unwrap();
        assert_eq!(rows.tracks.len(), 1);
        assert_eq!(rows.tracks[0].video_id.as_deref(), Some("abcdefghijk"));
        assert!(rows.tracks[0].path.is_none());
        let path: String = db
            .connection
            .query_row("SELECT path FROM tracks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(path, "youtube:abcdefghijk");
        let mut bad = s.clone();
        bad.video_id = "bad".into();
        let mut first = s;
        first.video_id = "zyxwvutsrqp".into();
        assert!(db.save_online(&[first, bad]).is_err());
        assert_eq!(db.snapshot().unwrap().tracks.len(), 1);
    }
}
