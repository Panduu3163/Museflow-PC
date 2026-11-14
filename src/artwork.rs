//! Small, bounded artwork cache. All file/network/decode work stays on its worker.
use crate::catalog::Track;
use std::{
    collections::{VecDeque, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime},
};
pub const SIDE: u32 = 256;
const INPUT: u64 = 4 * 1024 * 1024;
const DISK: u64 = 64 * 1024 * 1024;
const FILES: usize = 256;
const MEMORY: usize = 16;
const SIDECARS: [&str; 4] = ["cover.jpg", "cover.png", "folder.jpg", "folder.png"];
#[derive(Clone)]
pub struct Pixels {
    pub rgba: Vec<u8>,
    pub origin: &'static str,
}
pub fn decode(raw: &[u8], origin: &'static str) -> Option<Pixels> {
    if raw.is_empty() || raw.len() as u64 > INPUT {
        return None;
    }
    let reader = image::ImageReader::new(Cursor::new(raw))
        .with_guessed_format()
        .ok()?;
    if !matches!(
        reader.format(),
        Some(image::ImageFormat::Png | image::ImageFormat::Jpeg)
    ) {
        return None;
    }
    let (w, h) = reader.into_dimensions().ok()?;
    if w == 0 || h == 0 || w > 4096 || h > 4096 || u64::from(w) * u64::from(h) > 4_194_304 {
        return None;
    }
    let mut reader = image::ImageReader::new(Cursor::new(raw))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .ok()?
        .resize_to_fill(SIDE, SIDE, image::imageops::FilterType::Triangle)
        .into_rgba8();
    Some(Pixels {
        rgba: image.into_raw(),
        origin,
    })
}
fn read_small(path: &Path) -> Option<Vec<u8>> {
    let file = std::fs::File::open(path).ok()?;
    if file.metadata().ok()?.len() > INPUT {
        return None;
    }
    let mut raw = Vec::new();
    file.take(INPUT + 1).read_to_end(&mut raw).ok()?;
    (raw.len() as u64 <= INPUT).then_some(raw)
}
fn stamp(path: &Path, hash: &mut DefaultHasher) {
    path.hash(hash);
    if let Ok(m) = std::fs::metadata(path) {
        m.len().hash(hash);
        m.modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .hash(hash);
    }
}
fn key(track: &Track) -> Option<String> {
    if let Some(id) = &track.video_id {
        return crate::online::valid_id(id).then(|| format!("yt-{id}.png"));
    }
    let path = track.path.as_ref()?;
    let mut hash = DefaultHasher::new();
    stamp(path, &mut hash);
    if let Some(parent) = path.parent() {
        for name in SIDECARS {
            stamp(&parent.join(name), &mut hash);
        }
    }
    Some(format!("local-{:016x}.png", hash.finish()))
}
fn owned_name(name: &str) -> bool {
    if let Some(id) = name
        .strip_prefix("yt-")
        .and_then(|s| s.strip_suffix(".png"))
    {
        return crate::online::valid_id(id);
    }
    name.strip_prefix("local-")
        .and_then(|s| s.strip_suffix(".png"))
        .is_some_and(|s| s.len() == 16 && s.bytes().all(|b| b.is_ascii_hexdigit()))
}
fn cache_file(name: &str) -> bool {
    owned_name(name)
        || name.rsplit_once(".part-").is_some_and(|(key, pid)| {
            owned_name(key)
                && !pid.is_empty()
                && pid.len() <= 10
                && pid.bytes().all(|b| b.is_ascii_digit())
        })
}
fn local(track: &Track) -> Option<Pixels> {
    use lofty::file::TaggedFileExt;
    let path = track.path.as_ref()?;
    lofty::config::apply_global_options(
        lofty::config::GlobalOptions::new()
            .allocation_limit(INPUT as usize)
            .preserve_format_specific_items(false),
    );
    if let Ok(probe) = lofty::probe::Probe::open(path)
        && let Ok(file) = probe
            .options(
                lofty::config::ParseOptions::new()
                    .read_properties(false)
                    .read_cover_art(true),
            )
            .read()
    {
        for front in [true, false] {
            for tag in file.tags() {
                for picture in tag.pictures() {
                    if (picture.pic_type() == lofty::picture::PictureType::CoverFront) == front
                        && let Some(p) = decode(picture.data(), "embedded")
                    {
                        return Some(p);
                    }
                }
            }
        }
    }
    let parent = path.parent()?;
    for name in SIDECARS {
        if let Some(raw) = read_small(&parent.join(name))
            && let Some(p) = decode(&raw, "folder")
        {
            return Some(p);
        }
    }
    None
}
struct Entry {
    key: String,
    pixels: Option<Arc<Pixels>>,
    expires: Instant,
}
pub struct Cache {
    root: PathBuf,
    memory: VecDeque<Entry>,
    client: Result<reqwest::Client, reqwest::Error>,
    runtime: std::io::Result<tokio::runtime::Runtime>,
}
impl Cache {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            memory: VecDeque::new(),
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(3))
                .timeout(Duration::from_secs(6))
                .build(),
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build(),
        }
    }
    pub fn get(
        &mut self,
        track: &Track,
        network: bool,
        generation: &AtomicU64,
        ticket: u64,
    ) -> Option<Arc<Pixels>> {
        if generation.load(Ordering::Acquire) != ticket {
            return None;
        }
        let key = key(track)?;
        if let Some(index) = self
            .memory
            .iter()
            .position(|e| e.key == key && e.expires > Instant::now())
        {
            let item = self.memory.remove(index).unwrap();
            let pixels = item.pixels.clone();
            self.memory.push_back(item);
            return pixels;
        }
        let path = self.root.join(&key);
        let fresh = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age < Duration::from_secs(7 * 24 * 60 * 60));
        let cached = fresh
            .then(|| read_small(&path))
            .flatten()
            .and_then(|raw| decode(&raw, "cached"));
        let pixels = cached.or_else(|| {
            if generation.load(Ordering::Acquire) != ticket {
                return None;
            }
            if let Some(id) = &track.video_id {
                if !network {
                    return None;
                }
                self.download(id, generation, ticket)
                    .and_then(|raw| decode(&raw, "online"))
            } else {
                local(track)
            }
        });
        if generation.load(Ordering::Acquire) != ticket {
            return None;
        }
        if let Some(p) = &pixels
            && p.origin != "cached"
        {
            self.store(&key, p);
        }
        let pixels = pixels.map(Arc::new);
        // A cache-only startup miss must not suppress the subsequent Play request.
        if pixels.is_some() || network || track.video_id.is_none() {
            self.memory.retain(|e| e.key != key);
            self.memory.push_back(Entry {
                key,
                pixels: pixels.clone(),
                expires: Instant::now()
                    + Duration::from_secs(if pixels.is_some() { 600 } else { 60 }),
            });
            while self.memory.len() > MEMORY {
                self.memory.pop_front();
            }
        }
        pixels
    }
    fn download(&self, id: &str, generation: &AtomicU64, ticket: u64) -> Option<Vec<u8>> {
        if !crate::online::valid_id(id) {
            return None;
        }
        // Fixed YouTube image host; no cookies or arbitrary URL/redirect input.
        self.fetch_at(
            &format!("https://i.ytimg.com/vi/{id}/mqdefault.jpg"),
            generation,
            ticket,
        )
    }
    fn fetch_at(&self, url: &str, generation: &AtomicU64, ticket: u64) -> Option<Vec<u8>> {
        let client = self.client.as_ref().ok()?;
        self.runtime.as_ref().ok()?.block_on(async{
            let request=async{
                let mut response=client.get(url).send().await.ok()?;
                if !response.status().is_success()||response.content_length().is_some_and(|n|n>INPUT){return None;}
                let mut raw=Vec::new();
                while let Some(chunk)=response.chunk().await.ok()?{if raw.len()+chunk.len()>INPUT as usize{return None;}raw.extend_from_slice(&chunk);}
                Some(raw)
            };
            tokio::pin!(request);
            loop{tokio::select!{result=&mut request=>return result,_=tokio::time::sleep(Duration::from_millis(40))=>{if generation.load(Ordering::Acquire)!=ticket{return None;}}}}
        })
    }
    fn make_room(&self, reserve: u64) -> bool {
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return false;
        };
        let mut files = entries
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_ok_and(|t| t.is_file())
                    && cache_file(&e.file_name().to_string_lossy())
            })
            .filter_map(|e| {
                let m = e.metadata().ok()?;
                Some((
                    m.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                    m.len(),
                    e.path(),
                ))
            })
            .collect::<Vec<_>>();
        files.sort_by_key(|e| e.0);
        let mut count = files.len();
        let mut size = files.iter().map(|e| e.1).sum::<u64>();
        for (_, bytes, path) in files {
            if size + reserve <= DISK && count < FILES {
                break;
            }
            if std::fs::remove_file(path).is_ok() {
                size = size.saturating_sub(bytes);
                count -= 1;
            }
        }
        size + reserve <= DISK && count < FILES
    }
    fn store(&self, key: &str, pixels: &Pixels) {
        if !owned_name(key) || std::fs::create_dir_all(&self.root).is_err() {
            return;
        }
        let mut raw = Cursor::new(Vec::new());
        let Some(image) = image::RgbaImage::from_raw(SIDE, SIDE, pixels.rgba.clone()) else {
            return;
        };
        if image::DynamicImage::ImageRgba8(image)
            .write_to(&mut raw, image::ImageFormat::Png)
            .is_err()
        {
            return;
        }
        let bytes = raw.into_inner();
        if !self.make_room(bytes.len() as u64) {
            return;
        }
        let target = self.root.join(key);
        let temp = self.root.join(format!("{key}.part-{}", std::process::id()));
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
        {
            let written = file.write_all(&bytes).and_then(|_| file.flush()).is_ok();
            drop(file);
            if written {
                let _ = std::fs::remove_file(&target);
                let _ = std::fs::rename(&temp, &target);
            }
            let _ = std::fs::remove_file(temp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn http_failure_size_limit_and_inflight_cancellation_are_bounded() {
        use std::net::TcpListener;
        let root = directory();
        let mut cache = Cache::new(root.clone());
        cache.client = Ok(reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap());
        for (status, length, delay, cancel) in [
            ("200 OK", 8, 0, false),
            ("404 Not Found", 8, 0, false),
            ("200 OK", INPUT + 1, 0, false),
            ("200 OK", 8, 300, true),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let url = format!("http://{}/cover", listener.local_addr().unwrap());
            let server = std::thread::spawn(move || {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut input = [0; 4096];
                let _ = socket.read(&mut input);
                std::thread::sleep(Duration::from_millis(delay));
                let _ = write!(
                    socket,
                    "HTTP/1.1 {status}\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n12345678"
                );
            });
            let generation = Arc::new(AtomicU64::new(0));
            let g = generation.clone();
            let signal = cancel.then(|| {
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(50));
                    g.store(1, Ordering::Release);
                })
            });
            let start = Instant::now();
            let result = cache.fetch_at(&url, &generation, 0);
            assert_eq!(
                result.is_some(),
                status == "200 OK" && length <= INPUT && !cancel
            );
            if cancel {
                assert!(start.elapsed() < Duration::from_millis(250));
            }
            if let Some(signal) = signal {
                signal.join().unwrap();
            }
            server.join().unwrap();
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    fn png(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
        let image = image::RgbaImage::from_pixel(width, height, image::Rgba(color));
        let mut out = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut out, image::ImageFormat::Png)
            .unwrap();
        out.into_inner()
    }
    fn directory() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "museflow-art-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn decoder_rejects_broken_large_and_excessive_dimensions() {
        assert!(decode(b"not an image", "test").is_none());
        assert!(decode(&vec![0; INPUT as usize + 1], "test").is_none());
        assert!(decode(&png(4097, 1, [0, 0, 0, 255]), "test").is_none());
        let pixels = decode(&png(4, 8, [220, 30, 60, 255]), "test").unwrap();
        assert_eq!(pixels.rgba.len(), SIDE as usize * SIDE as usize * 4);
        assert_eq!(&pixels.rgba[..4], &[220, 30, 60, 255]);
        let mut jpeg = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            8,
            8,
            image::Rgb([25, 75, 200]),
        ))
        .write_to(&mut jpeg, image::ImageFormat::Jpeg)
        .unwrap();
        assert!(decode(&jpeg.into_inner(), "test").is_some());
    }
    #[test]
    fn folder_cache_persists_invalidates_and_missing_entries_stay_bounded() {
        let root = directory();
        let media = root.join("song.wav");
        std::fs::write(&media, b"invalid media; folder artwork still works").unwrap();
        let cover = root.join("cover.png");
        std::fs::write(&cover, png(2, 2, [20, 90, 210, 255])).unwrap();
        let track = Track::local(
            1,
            media,
            "Song".into(),
            "Artist".into(),
            "Album".into(),
            1.0,
        );
        let generation = AtomicU64::new(0);
        let mut cache = Cache::new(root.join("cache"));
        let p = cache.get(&track, false, &generation, 0).unwrap();
        assert_eq!(p.origin, "folder");
        assert!(p.rgba[2] > p.rgba[0]);
        let mut cache = Cache::new(root.join("cache"));
        let p = cache.get(&track, false, &generation, 0).unwrap();
        assert_eq!(p.origin, "cached");
        assert!(p.rgba[2] > p.rgba[0]);
        let old = key(&track).unwrap();
        std::fs::write(&cover, png(3, 4, [210, 30, 70, 255])).unwrap();
        assert_ne!(old, key(&track).unwrap());
        let p = cache.get(&track, false, &generation, 0).unwrap();
        assert!(p.rgba[0] > p.rgba[2]);
        for n in 0..32 {
            let track = Track::local(
                n,
                root.join(format!("missing-{n}/no.wav")),
                "".into(),
                "".into(),
                "".into(),
                0.0,
            );
            assert!(cache.get(&track, false, &generation, 0).is_none());
        }
        assert_eq!(cache.memory.len(), MEMORY);
        assert!(cache.get(&track, false, &AtomicU64::new(1), 0).is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn quota_eviction_only_removes_owned_flat_cache_files() {
        let root = directory();
        let cache = Cache::new(root.clone());
        std::fs::write(root.join("notes.txt"), b"leave this alone").unwrap();
        std::fs::create_dir(root.join("yt-abcdefghijk.png")).unwrap();
        std::fs::write(root.join("local-0000000000000001.png"), b"cache").unwrap();
        std::fs::write(
            root.join("local-0000000000000001.png.part-123"),
            b"interrupted cache write",
        )
        .unwrap();
        assert!(cache.make_room(DISK));
        assert!(root.join("notes.txt").exists());
        assert!(root.join("yt-abcdefghijk.png").is_dir());
        assert!(!root.join("local-0000000000000001.png").exists());
        assert!(!root.join("local-0000000000000001.png.part-123").exists());
        assert!(!cache.make_room(DISK + 1));
        assert!(!owned_name("../escape.png"));
        for n in 0..FILES {
            std::fs::write(root.join(format!("local-{n:016x}.png")), b"cache").unwrap();
        }
        assert!(cache.make_room(1));
        assert_eq!(
            std::fs::read_dir(&root)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().unwrap().is_file()
                    && owned_name(&e.file_name().to_string_lossy()))
                .count(),
            FILES - 1
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
