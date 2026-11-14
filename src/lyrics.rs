//! On-demand sidecar/LRCLIB lyrics. Provider text is never used as application instructions.
use crate::catalog::Track;
use serde_json::Value;
use std::{
    collections::VecDeque,
    io::Read,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
const LIMIT: usize = 128 * 1024;
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub at: Option<f64>,
    pub text: String,
}
#[derive(Clone, Debug)]
pub struct Document {
    pub lines: Vec<Line>,
    pub source: String,
    pub status: String,
}
impl Document {
    pub fn message(status: &str) -> Self {
        Self {
            lines: Vec::new(),
            source: String::new(),
            status: status.into(),
        }
    }
    pub fn current(&self, position: f64) -> i32 {
        if !position.is_finite() || self.lines.first().is_none_or(|l| l.at.is_none()) {
            return -1;
        }
        self.lines
            .partition_point(|l| l.at.is_some_and(|t| t <= position)) as i32
            - 1
    }
}
fn timestamp(s: &str) -> Option<f64> {
    let (m, s) = s.split_once(':')?;
    let m = m.parse::<u32>().ok()?;
    let s = s.parse::<f64>().ok()?;
    if !s.is_finite() || !(0.0..60.0).contains(&s) || m > 1440 {
        return None;
    }
    Some(m as f64 * 60.0 + s)
}
pub fn parse(text: &str, source: &str) -> Result<Document, String> {
    if text.len() > LIMIT {
        return Err("Lyrics file is too large (128 KiB maximum).".into());
    }
    let mut timed = Vec::new();
    let mut plain = Vec::new();
    let mut offset = 0.0;
    for raw in text.trim_start_matches('\u{feff}').lines().take(2000) {
        let line = raw.trim();
        let mut rest = line;
        let mut times = Vec::new();
        let mut metadata = false;
        while let Some(tag) = rest.strip_prefix('[').and_then(|s| s.split_once(']')) {
            if let Some(n) = timestamp(tag.0) {
                times.push(n);
            } else {
                metadata = tag.0.contains(':');
                if let Some(n) = tag
                    .0
                    .strip_prefix("offset:")
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    offset = (n as f64 / 1000.0).clamp(-60.0, 60.0);
                }
            }
            rest = tag.1;
        }
        let body = rest
            .chars()
            .filter(|c| !c.is_control() || *c == '\t')
            .take(2000)
            .collect::<String>();
        if !times.is_empty() {
            for t in times {
                if timed.len() < 2000 {
                    timed.push(Line {
                        at: Some(t),
                        text: body.clone(),
                    });
                }
            }
        } else if !metadata {
            plain.push(Line {
                at: None,
                text: body,
            });
        }
    }
    if !timed.is_empty() {
        for l in &mut timed {
            l.at = Some((l.at.unwrap() + offset).max(0.0));
        }
        timed.sort_by(|a, b| a.at.unwrap().total_cmp(&b.at.unwrap()));
        timed.dedup_by(|a, b| a.at == b.at && a.text == b.text);
    }
    let mut lines = if timed.is_empty() { plain } else { timed };
    let mut bytes = 0;
    lines.retain(|l| {
        bytes += l.text.len();
        bytes <= LIMIT
    });
    if lines.iter().all(|l| l.text.is_empty()) {
        return Ok(Document::message(
            "No lyrics are available for this recording.",
        ));
    }
    Ok(Document {
        lines,
        source: source.into(),
        status: String::new(),
    })
}
pub fn sidecar(track: &Track) -> Option<Result<Document, String>> {
    let file = track.path.as_ref()?;
    for ext in ["lrc", "txt"] {
        let path = file.with_extension(ext);
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if !meta.is_file() || meta.file_type().is_symlink() {
            continue;
        }
        return Some((|| {
            if meta.len() > LIMIT as u64 {
                return Err("Lyrics file is too large (128 KiB maximum).".into());
            }
            let file = std::fs::File::open(&path).map_err(|_| "Cannot read this lyrics sidecar")?;
            let mut data = Vec::new();
            file.take((LIMIT + 1) as u64)
                .read_to_end(&mut data)
                .map_err(|_| "Cannot read this lyrics sidecar")?;
            let text =
                String::from_utf8(data).map_err(|_| "Lyrics sidecars must use UTF-8 text")?;
            parse(&text, "Local lyrics sidecar")
        })());
    }
    None
}
fn normalized(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
pub fn from_record(value: &Value, track: &Track) -> Result<Document, String> {
    let title = value["trackName"].as_str().unwrap_or_default();
    let artist = value["artistName"].as_str().unwrap_or_default();
    let duration = value["duration"]
        .as_f64()
        .filter(|n| n.is_finite())
        .ok_or("The lyrics provider returned invalid metadata")?;
    if normalized(title) != normalized(&track.title)
        || normalized(artist) != normalized(&track.artist)
        || (track.seconds > 0.0 && (duration - track.seconds).abs() > 3.0)
    {
        return Err(
            "The lyrics match belongs to a different recording. No lyrics were loaded.".into(),
        );
    }
    if value["instrumental"].as_bool() == Some(true) {
        return Ok(Document::message("Instrumental · no lyrics"));
    }
    let synced = value["syncedLyrics"]
        .as_str()
        .filter(|s| !s.trim().is_empty());
    let plain = value["plainLyrics"].as_str().unwrap_or_default();
    parse(synced.unwrap_or(plain), "Lyrics from LRCLIB")
}
pub struct Client {
    client: reqwest::Client,
    runtime: tokio::runtime::Runtime,
    cooldown: Instant,
    cache: VecDeque<(String, Document)>,
    endpoint: String,
}
impl Client {
    fn key(track: &Track) -> String {
        format!(
            "{}\0{}\0{}\0{}",
            track.title,
            track.artist,
            track.album,
            track.seconds.round()
        )
    }
    pub fn forget(&mut self, track: &Track) {
        let key = Self::key(track);
        self.cache.retain(|(k, _)| *k != key);
    }
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            client: reqwest::Client::builder()
                .user_agent("MuseFlowPC/0.4.0 (https://github.com/Panduu3163/Muse_Flow)")
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(12))
                .build()
                .map_err(|_| "Cannot initialize lyrics networking")?,
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|_| "Cannot start the lyrics worker")?,
            cooldown: Instant::now(),
            cache: VecDeque::new(),
            endpoint: "https://lrclib.net/api/get".into(),
        })
    }
    pub fn lookup(
        &mut self,
        track: &Track,
        generation: Arc<AtomicU64>,
        ticket: u64,
    ) -> Result<Document, String> {
        if let Some(local) = sidecar(track) {
            return local;
        }
        if track.artist.trim().is_empty()
            || track.artist == "Unknown artist"
            || track.title.trim().is_empty()
        {
            return Ok(Document::message(
                "Add title and artist tags, or place a matching .lrc/.txt file beside this song.",
            ));
        }
        let key = Self::key(track);
        if let Some(n) = self.cache.iter().position(|(k, _)| *k == key) {
            let value = self.cache.remove(n).unwrap();
            let doc = value.1.clone();
            self.cache.push_back(value);
            return Ok(doc);
        }
        if self.cooldown.saturating_duration_since(Instant::now()) > Duration::from_millis(500) {
            return Err("Lyrics lookup is cooling down. Try again later.".into());
        }
        while Instant::now() < self.cooldown {
            if generation.load(Ordering::Acquire) != ticket {
                return Err("Cancelled".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let result=self.runtime.block_on(async{
            let request=async{
                let mut url=reqwest::Url::parse(&self.endpoint).map_err(|_|("Invalid lyrics service address".to_owned(),Duration::from_millis(300)))?;
                url.query_pairs_mut().append_pair("track_name",&track.title).append_pair("artist_name",&track.artist);
                if !track.album.is_empty()&&track.album!="Local music"{url.query_pairs_mut().append_pair("album_name",&track.album);}
                if (1.0..=3600.0).contains(&track.seconds){url.query_pairs_mut().append_pair("duration",&track.seconds.round().to_string());}
                let mut response=self.client.get(url).send().await.map_err(|_|("Cannot reach the lyrics service. Check your connection and retry.".to_owned(),Duration::from_millis(300)))?;
                let status=response.status().as_u16();
                if status==429||status==503{let delay=response.headers().get("retry-after").and_then(|v|v.to_str().ok()).and_then(|v|v.parse::<u64>().ok()).unwrap_or(60).max(1);return Err(("Lyrics service is busy. Please retry later.".into(),Duration::from_secs(delay.saturating_add(1))));}
                if status==404{return Ok(Document::message("No lyrics found for this recording."));}
                if !response.status().is_success(){return Err((format!("Lyrics lookup failed (HTTP {status}). Try again later."),Duration::from_millis(300)));}
                let mut bytes=Vec::new();while let Some(chunk)=response.chunk().await.map_err(|_|("Lyrics lookup was interrupted.".to_owned(),Duration::from_millis(300)))?{if bytes.len()+chunk.len()>512*1024{return Err(("Lyrics response was too large.".into(),Duration::from_millis(300)));}bytes.extend_from_slice(&chunk);}
                let value=serde_json::from_slice(&bytes).map_err(|_|("Lyrics provider returned invalid data.".to_owned(),Duration::from_millis(300)))?;
                from_record(&value,track).map_err(|e|(e,Duration::from_millis(300)))
            };tokio::pin!(request);
            loop{tokio::select!{r=&mut request=>return r,_=tokio::time::sleep(Duration::from_millis(40))=>{if generation.load(Ordering::Acquire)!=ticket{return Err(("Cancelled".into(),Duration::from_millis(300)));}}}}
        });
        match result {
            Ok(doc) => {
                self.cooldown = Instant::now() + Duration::from_millis(300);
                self.cache.push_back((key, doc.clone()));
                while self.cache.len() > 24 {
                    self.cache.pop_front();
                }
                Ok(doc)
            }
            Err((e, wait)) => {
                self.cooldown = Instant::now()
                    .checked_add(wait)
                    .unwrap_or_else(|| Instant::now() + Duration::from_secs(86400));
                Err(e)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    fn track() -> Track {
        Track::local(
            1,
            Path::new("x.wav").into(),
            "Test song".into(),
            "Test artist".into(),
            "".into(),
            120.0,
        )
    }
    #[test]
    fn lrc_multiple_times_offsets_unicode_and_seek() {
        let d = parse(
            "[ar:Test]\n[offset:-500]\n[00:02.00][00:04.50]হ্যালো\n[00:01.50]First line",
            "test",
        )
        .unwrap();
        assert_eq!(d.lines.len(), 3);
        assert_eq!(d.current(0.0), -1);
        assert_eq!(d.current(1.2), 0);
        assert_eq!(d.current(5.0), 2);
        assert_eq!(d.current(1.6), 1);
        assert_eq!(d.lines[1].text, "হ্যালো");
        assert_eq!(
            parse("Just text\nSecond line", "test")
                .unwrap()
                .current(100.0),
            -1
        );
    }
    #[test]
    fn strict_match_instrumental_and_size_limit() {
        let mut v = serde_json::json!({"trackName":"Test song","artistName":"Test artist","duration":120,"instrumental":true});
        assert!(
            from_record(&v, &track())
                .unwrap()
                .status
                .contains("Instrumental")
        );
        v["duration"] = 160.into();
        assert!(from_record(&v, &track()).is_err());
        assert!(parse(&"x".repeat(LIMIT + 1), "test").is_err());
        assert!(timestamp("00:99").is_none());
    }
    fn server(
        status: &str,
        headers: &str,
        body: Vec<u8>,
        delay: Duration,
    ) -> (String, std::thread::JoinHandle<()>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (status, headers) = (status.to_owned(), headers.to_owned());
        let thread = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut buffer = [0; 4096];
            let _ = stream.read(&mut buffer);
            std::thread::sleep(delay);
            let header = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\n{headers}Connection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&body);
        });
        (format!("http://{address}/get"), thread)
    }
    #[test]
    fn cached_lyrics_do_not_repeat_network_requests_and_rate_limits_are_honoured() {
        let body=serde_json::to_vec(&serde_json::json!({"trackName":"Test song","artistName":"Test artist","duration":120,"plainLyrics":"Generated test line"})).unwrap();
        let (url, server) = server("200 OK", "", body, Duration::ZERO);
        let mut c = Client::new().unwrap();
        c.endpoint = url;
        let g = Arc::new(AtomicU64::new(0));
        assert_eq!(
            c.lookup(&track(), g.clone(), 0).unwrap().lines[0].text,
            "Generated test line"
        );
        server.join().unwrap();
        assert_eq!(c.lookup(&track(), g, 0).unwrap().lines.len(), 1);
        let (url, server) = self::server(
            "429 Too Many Requests",
            "Retry-After: 3600\r\n",
            Vec::new(),
            Duration::ZERO,
        );
        let mut c = Client::new().unwrap();
        c.endpoint = url;
        let g = Arc::new(AtomicU64::new(0));
        assert!(c.lookup(&track(), g.clone(), 0).is_err());
        assert!(c.cooldown.saturating_duration_since(Instant::now()) >= Duration::from_secs(3600));
        assert!(
            c.lookup(&track(), g, 0)
                .err()
                .unwrap()
                .contains("cooling down")
        );
        server.join().unwrap();
    }
    #[test]
    fn lyrics_failures_are_bounded_and_cancellable() {
        for (body, message) in [
            (b"not json".to_vec(), "invalid data"),
            (vec![b'x'; 512 * 1024 + 1], "too large"),
        ] {
            let (url, t) = server("200 OK", "", body, Duration::ZERO);
            let mut c = Client::new().unwrap();
            c.endpoint = url;
            assert!(
                c.lookup(&track(), Arc::new(AtomicU64::new(0)), 0)
                    .err()
                    .unwrap()
                    .contains(message)
            );
            t.join().unwrap();
        }
        let (url, t) = server("200 OK", "", b"{}".to_vec(), Duration::from_millis(300));
        let mut c = Client::new().unwrap();
        c.endpoint = url;
        let g = Arc::new(AtomicU64::new(0));
        let changed = g.clone();
        let cancel = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            changed.store(1, Ordering::Release);
        });
        let start = Instant::now();
        assert_eq!(c.lookup(&track(), g, 0).err().unwrap(), "Cancelled");
        assert!(start.elapsed() < Duration::from_secs(1));
        cancel.join().unwrap();
        t.join().unwrap();
    }
}
