//! Minimal music catalog port informed by the attributed Android InnerTube/YouTube
//! references (including Echo-Music adaptations). No account cookies or private API keys.
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Song {
    pub video_id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub seconds: f64,
}
#[derive(Clone, Default)]
pub struct Page {
    pub songs: Vec<Song>,
    pub continuation: Option<String>,
}
pub fn valid_id(id: &str) -> bool {
    id.len() == 11
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn text(v: &Value) -> String {
    v.get("runs")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|r| r["text"].as_str()).collect())
        .or_else(|| v["simpleText"].as_str().map(str::to_owned))
        .unwrap_or_default()
}
fn clean(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).take(512).collect()
}
fn seconds(s: &str) -> Option<f64> {
    let parts = s.split(':').collect::<Vec<_>>();
    if !(2..=3).contains(&parts.len()) {
        return None;
    }
    let mut n = 0u32;
    for (i, p) in parts.iter().enumerate() {
        let x = p.parse::<u32>().ok()?;
        if i > 0 && x >= 60 {
            return None;
        }
        n = n.checked_mul(60)?.checked_add(x)?;
    }
    Some(n as f64)
}
fn item(v: &Value) -> Option<Song> {
    let cols = v["flexColumns"].as_array()?;
    let title = cols.first()?["musicResponsiveListItemFlexColumnRenderer"]["text"].clone();
    let id = title["runs"]
        .as_array()?
        .iter()
        .find_map(|r| r["navigationEndpoint"]["watchEndpoint"]["videoId"].as_str())?;
    if !valid_id(id) {
        return None;
    }
    let runs =
        cols.get(1)?["musicResponsiveListItemFlexColumnRenderer"]["text"]["runs"].as_array()?;
    let mut artists = Vec::new();
    let mut album = String::new();
    let mut duration = None;
    for r in runs {
        let t = r["text"].as_str().unwrap_or_default();
        let browse = &r["navigationEndpoint"]["browseEndpoint"];
        let kind=browse["browseEndpointContextSupportedConfigs"]["browseEndpointContextMusicConfig"]["pageType"].as_str().unwrap_or_default();
        if kind == "MUSIC_PAGE_TYPE_ARTIST" {
            artists.push(t);
        }
        if kind == "MUSIC_PAGE_TYPE_ALBUM" {
            album = clean(t);
        }
        duration = duration.or_else(|| seconds(t));
    }
    if let Some(fixed) = v["fixedColumns"].as_array() {
        for c in fixed {
            duration = duration.or_else(|| {
                seconds(&text(
                    &c["musicResponsiveListItemFixedColumnRenderer"]["text"],
                ))
            });
        }
    }
    let title = clean(&text(&title));
    if title.is_empty() {
        return None;
    }
    Some(Song {
        video_id: id.into(),
        title,
        artist: if artists.is_empty() {
            "Unknown artist".into()
        } else {
            clean(&artists.join(", "))
        },
        album,
        seconds: duration.unwrap_or(0.0),
    })
}
pub fn parse_page(value: &Value) -> Result<Page, String> {
    fn walk(v: &Value, out: &mut Page, seen: &mut BTreeSet<String>, depth: usize) {
        if depth > 64 || out.songs.len() >= 100 {
            return;
        }
        match v {
            Value::Object(o) => {
                if let Some(x) = o.get("musicResponsiveListItemRenderer") {
                    if let Some(song) = item(x)
                        && seen.insert(song.video_id.clone())
                    {
                        out.songs.push(song);
                    }
                    return;
                }
                if let Some(s) = o
                    .get("nextContinuationData")
                    .and_then(|d| d["continuation"].as_str())
                    .filter(|s| s.len() <= 16384)
                {
                    out.continuation = Some(s.into());
                }
                if let Some(s) = o
                    .get("continuationCommand")
                    .and_then(|d| d["token"].as_str())
                    .filter(|s| s.len() <= 16384)
                {
                    out.continuation = Some(s.into());
                }
                for x in o.values() {
                    walk(x, out, seen, depth + 1);
                }
            }
            Value::Array(a) => {
                for x in a {
                    walk(x, out, seen, depth + 1);
                }
            }
            _ => {}
        }
    }
    if value.get("error").is_some() {
        return Err("The music catalog rejected the request. Try again later.".into());
    }
    let mut out = Page::default();
    walk(value, &mut out, &mut BTreeSet::new(), 0);
    if out.songs.is_empty()
        && value.get("contents").is_none()
        && value.get("continuationContents").is_none()
    {
        return Err("The music catalog returned an unfamiliar response. Try again later.".into());
    }
    Ok(out)
}
pub struct Catalog {
    client: reqwest::Client,
    runtime: tokio::runtime::Runtime,
}
impl Catalog {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 MuseFlowPC/0.3.0")
                .connect_timeout(Duration::from_secs(8))
                .timeout(Duration::from_secs(20))
                .build()
                .map_err(|_| "Cannot initialize network access")?,
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|_| "Cannot initialize network worker")?,
        })
    }
    pub fn search(
        &self,
        query: &str,
        continuation: Option<&str>,
        generation: Arc<AtomicU64>,
        ticket: u64,
    ) -> Result<Page, String> {
        self.search_at(
            query,
            continuation,
            generation,
            ticket,
            "https://music.youtube.com/youtubei/v1/search",
        )
    }
    fn search_at(
        &self,
        query: &str,
        continuation: Option<&str>,
        generation: Arc<AtomicU64>,
        ticket: u64,
        endpoint: &str,
    ) -> Result<Page, String> {
        self.runtime.block_on(async {
            let request=async {
                let mut url=reqwest::Url::parse(endpoint).map_err(|_|"Invalid catalog endpoint".to_owned())?;
                url.query_pairs_mut().append_pair("prettyPrint","false");
                if let Some(c)=continuation{url.query_pairs_mut().append_pair("continuation",c).append_pair("ctoken",c);}
                let body=json!({"context":{"client":{"clientName":"WEB_REMIX","clientVersion":"1.20260213.01.00","hl":"en","gl":"US"}},"query":query,"params":"EgWKAQIIAWoKEAkQBRAKEAMQBA%3D%3D"});
                let mut r=self.client.post(url).header("Origin","https://music.youtube.com").header("X-YouTube-Client-Name","67").header("X-YouTube-Client-Version","1.20260213.01.00").json(&body).send().await.map_err(|_|"Cannot reach the music catalog. Check your connection and try again.".to_owned())?;
                if !r.status().is_success(){return Err(format!("Music search is unavailable (HTTP {}). Try again later.",r.status().as_u16()));}
                let mut data=Vec::new();while let Some(chunk)=r.chunk().await.map_err(|_|"Music search was interrupted. Try again.".to_owned())?{if data.len()+chunk.len()>2*1024*1024{return Err("Music search response was too large.".into());}data.extend_from_slice(&chunk);}
                let v=serde_json::from_slice(&data).map_err(|_|"Music search returned invalid data.".to_owned())?;parse_page(&v)
            };
            tokio::pin!(request);
            loop {tokio::select!{result=&mut request=>return result,_=tokio::time::sleep(Duration::from_millis(40))=>{if generation.load(Ordering::Acquire)!=ticket{return Err("Cancelled".into());}}}}
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(id: &str) -> Value {
        json!({"musicResponsiveListItemRenderer":{"flexColumns":[{"musicResponsiveListItemFlexColumnRenderer":{"text":{"runs":[{"text":"A song","navigationEndpoint":{"watchEndpoint":{"videoId":id}}}]}}},{"musicResponsiveListItemFlexColumnRenderer":{"text":{"runs":[{"text":"Singer","navigationEndpoint":{"browseEndpoint":{"browseEndpointContextSupportedConfigs":{"browseEndpointContextMusicConfig":{"pageType":"MUSIC_PAGE_TYPE_ARTIST"}}}}},{"text":"3:14"}]}}}]}})
    }
    #[test]
    fn parses_identity_dedup_duration_and_continuation() {
        let v = json!({"contents":[row("abcdefghijk"),row("abcdefghijk"),row("bad"),{"nextContinuationData":{"continuation":"next"}}]});
        let p = parse_page(&v).unwrap();
        assert_eq!(p.songs.len(), 1);
        assert_eq!(p.songs[0].artist, "Singer");
        assert_eq!(p.songs[0].seconds, 194.0);
        assert_eq!(p.continuation.as_deref(), Some("next"));
    }
    #[test]
    fn distinguishes_no_results_from_bad_schema() {
        assert!(
            parse_page(&json!({"contents":{}}))
                .unwrap()
                .songs
                .is_empty()
        );
        assert!(parse_page(&json!({})).is_err());
        assert!(parse_page(&json!({"error":{"code":403}})).is_err());
        assert!(!valid_id("../../x"));
        assert_eq!(seconds("1:99"), None);
    }
    fn server(
        status: &str,
        body: Vec<u8>,
        delay: Duration,
    ) -> (String, std::thread::JoinHandle<()>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let status = status.to_owned();
        let thread = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut buffer = [0; 4096];
            let _ = stream.read(&mut buffer);
            std::thread::sleep(delay);
            let header = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&body);
        });
        (format!("http://{address}/search"), thread)
    }
    #[test]
    fn network_errors_malformed_and_oversized_responses_are_recoverable() {
        let c = Catalog::new().unwrap();
        for (status, body, expected) in [
            ("503 Unavailable", b"temporary".to_vec(), "503"),
            ("200 OK", b"invalid".to_vec(), "invalid data"),
            ("200 OK", vec![b' '; 2 * 1024 * 1024 + 1], "too large"),
        ] {
            let (url, t) = server(status, body, Duration::ZERO);
            let result = c.search_at("test", None, Arc::new(AtomicU64::new(0)), 0, &url);
            assert!(result.err().unwrap().contains(expected));
            t.join().unwrap();
        }
    }
    #[test]
    fn cancelling_inflight_http_drops_the_request_promptly() {
        let (url, server) = server(
            "200 OK",
            br#"{"contents":{}}"#.to_vec(),
            Duration::from_millis(300),
        );
        let c = Catalog::new().unwrap();
        let generation = Arc::new(AtomicU64::new(0));
        let g = generation.clone();
        let change = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            g.store(1, Ordering::Release);
        });
        let start = std::time::Instant::now();
        assert_eq!(
            c.search_at("test", None, generation, 0, &url)
                .err()
                .unwrap(),
            "Cancelled"
        );
        assert!(start.elapsed() < Duration::from_secs(1));
        change.join().unwrap();
        server.join().unwrap();
    }
}
