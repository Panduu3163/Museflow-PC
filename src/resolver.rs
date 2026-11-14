//! On-demand resolver. No shell, cookies, plugins, cache or remote component loading.
use crate::online::valid_id;
use serde_json::Value;
use std::{
    io::Read,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::{Duration, Instant},
};
#[derive(Clone, Debug)]
pub struct Stream {
    pub url: String,
    pub headers: Vec<(String, String)>,
}
#[derive(Clone)]
pub struct Tools {
    pub resolver: PathBuf,
    pub javascript: PathBuf,
}
impl Tools {
    pub fn discover() -> Self {
        let base = std::env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_owned();
        Self {
            resolver: std::env::var_os("MUSEFLOW_YTDLP")
                .map(PathBuf::from)
                .unwrap_or_else(|| base.join("resolver/yt-dlp.exe")),
            javascript: std::env::var_os("MUSEFLOW_DENO")
                .map(PathBuf::from)
                .unwrap_or_else(|| base.join("javascript/deno.exe")),
        }
    }
}
pub fn parse(raw: &[u8], expected: &str) -> Result<Stream, String> {
    let v: Value =
        serde_json::from_slice(raw).map_err(|_| "The stream resolver returned invalid data.")?;
    if v["id"].as_str() != Some(expected) {
        return Err("The resolver returned a different recording. Playback stopped.".into());
    }
    let url = v["url"]
        .as_str()
        .ok_or("No playable audio stream is available for this recording.")?;
    let u = reqwest::Url::parse(url).map_err(|_| "Invalid audio stream address")?;
    if !audio_host(&u) {
        return Err("The resolver returned an unexpected audio host.".into());
    }
    if v["acodec"].as_str() == Some("none") {
        return Err("This recording has no playable audio.".into());
    }
    let mut headers = Vec::new();
    if let Some(o) = v["http_headers"].as_object() {
        for (k, v) in o {
            let s = v.as_str().ok_or("Invalid stream headers")?;
            if k.is_empty()
                || k.len() > 128
                || !k.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                || s.len() > 8192
                || s.chars().any(|c| c.is_control())
            {
                return Err("Invalid stream headers".into());
            }
            if matches!(
                k.to_ascii_lowercase().as_str(),
                "host" | "cookie" | "authorization" | "proxy-authorization"
            ) {
                return Err("Unexpected private stream headers".into());
            }
            headers.push((k.clone(), s.to_owned()));
        }
    }
    Ok(Stream {
        url: url.into(),
        headers,
    })
}
#[cfg(windows)]
struct Job(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl Job {
    fn attach(child: &Child) -> Result<Self, String> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::{Foundation::CloseHandle, System::JobObjects::*};
        // SAFETY: owned job handle, valid zeroed settings and child handle; no UI/system-global changes.
        unsafe {
            let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if h.is_null() {
                return Err("Cannot isolate the stream resolver".into());
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                h,
                JobObjectExtendedLimitInformation,
                (&info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&info) as u32,
            ) == 0
                || AssignProcessToJobObject(h, child.as_raw_handle()) == 0
            {
                CloseHandle(h);
                return Err("Cannot isolate the stream resolver".into());
            }
            Ok(Self(h))
        }
    }
}
#[cfg(windows)]
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}
fn read_bounded(mut input: impl Read, limit: u64, overflow: Arc<AtomicBool>) -> Vec<u8> {
    let mut out = Vec::new();
    let _ = input.by_ref().take(limit + 1).read_to_end(&mut out);
    if out.len() as u64 > limit {
        overflow.store(true, Ordering::Release);
        out.truncate(limit as usize);
    }
    out
}
struct Scratch {
    path: PathBuf,
    root: PathBuf,
}
impl Scratch {
    fn new(tools: &Tools) -> Result<Self, String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = tools
            .resolver
            .parent()
            .ok_or("Invalid resolver location")?
            .join("temp");
        std::fs::create_dir_all(&root)
            .map_err(|_| "Cannot create the resolver temporary folder")?;
        let root = root
            .canonicalize()
            .map_err(|_| "Cannot locate the resolver temporary folder")?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Invalid system time")?
            .as_nanos();
        let path = root.join(format!(
            "task-{}-{now}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).map_err(|_| "Cannot create resolver scratch space")?;
        Ok(Self { path, root })
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        // Only remove this invocation's newly created, canonical child directory.
        if let Ok(path) = self.path.canonicalize()
            && path.parent() == Some(self.root.as_path())
            && path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("task-"))
        {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}
pub fn resolve(
    tools: &Tools,
    id: &str,
    generation: &AtomicU64,
    ticket: u64,
) -> Result<Stream, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Cannot initialize stream verification")?;
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(6))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() < 4 && audio_host(attempt.url()) {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .build()
        .map_err(|_| "Cannot initialize stream verification")?;
    // Successful extraction does not establish that the CDN accepts the URL.
    // Preserve best audio first; only a refused/expired URL selects compatibility audio.
    for compatibility in [false, true] {
        if generation.load(Ordering::Acquire) != ticket {
            return Err("Cancelled".into());
        }
        let stream = resolve_once(tools, id, generation, ticket, compatibility)?;
        let checked = runtime.block_on(async {
            let request = verify_stream(&client, &stream);
            tokio::pin!(request);
            loop {
                tokio::select! {
                    result = &mut request => break result,
                    _ = tokio::time::sleep(Duration::from_millis(30)) => {
                        if generation.load(Ordering::Acquire) != ticket {
                            break Err(ProbeError::Other("Cancelled".into()));
                        }
                    }
                }
            }
        });
        match checked {
            Ok(()) => return Ok(stream),
            Err(ProbeError::Refused) if !compatibility => continue,
            Err(ProbeError::Refused) => return Err("YouTube refused both audio sources for this recording (HTTP 403/410). Try another recording or retry later.".into()),
            Err(ProbeError::Other(message)) => return Err(message),
        }
    }
    unreachable!()
}
fn audio_host(url: &reqwest::Url) -> bool {
    url.scheme() == "https"
        && url
            .host_str()
            .is_some_and(|h| h.ends_with(".googlevideo.com"))
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none_or(|p| p == 443)
}
#[derive(Debug, PartialEq)]
enum ProbeError {
    Refused,
    Other(String),
}
async fn verify_stream(client: &reqwest::Client, stream: &Stream) -> Result<(), ProbeError> {
    let length = verify_range(client, stream, 0).await?;
    // Some preview-only URLs serve a short prefix but refuse the rest of a song.
    // Verify a later range when Content-Range establishes that it exists.
    if length.is_some_and(|n| n > 1_064_960) {
        verify_range(client, stream, 1_048_576).await?;
    }
    Ok(())
}
async fn verify_range(
    client: &reqwest::Client,
    stream: &Stream,
    start: u64,
) -> Result<Option<u64>, ProbeError> {
    let mut request = client.get(&stream.url);
    for (key, value) in &stream.headers {
        request = request.header(key, value);
    }
    let mut response = request
        .header(
            reqwest::header::RANGE,
            format!("bytes={start}-{}", start + 16_383),
        )
        .send()
        .await
        .map_err(|_| {
            ProbeError::Other(
                "Could not reach the audio server. Check your connection and retry.".into(),
            )
        })?;
    match response.status().as_u16() {
        403 | 410 => return Err(ProbeError::Refused),
        200 | 206 => {}
        code => {
            return Err(ProbeError::Other(format!(
                "The audio server returned HTTP {code}. Retry later or choose another recording."
            )));
        }
    }
    let length = response
        .headers()
        .get(reqwest::header::CONTENT_RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.rsplit('/').next())
        .and_then(|v| v.parse::<u64>().ok());
    // Read only the first available chunk, then drop the response. Never download a track here.
    match response.chunk().await {
        Ok(Some(chunk)) if !chunk.is_empty() => Ok(length),
        _ => Err(ProbeError::Other(
            "The audio server returned no audio data. Retry or choose another recording.".into(),
        )),
    }
}
fn resolve_once(
    tools: &Tools,
    id: &str,
    generation: &AtomicU64,
    ticket: u64,
    compatibility: bool,
) -> Result<Stream, String> {
    if !valid_id(id) {
        return Err("Invalid recording identifier".into());
    }
    if !tools.resolver.is_file() || !tools.javascript.is_file() {
        return Err("Online playback needs the bundled resolver and JavaScript runtime. Restore those preview folders and try again.".into());
    }
    let scratch = Scratch::new(tools)?;
    let mut command = Command::new(&tools.resolver);
    command
        .env("TEMP", &scratch.path)
        .env("TMP", &scratch.path)
        .env("DENO_DIR", scratch.path.join("deno"));
    if compatibility {
        // The automatic TV client can return signed URLs that are refused for music.
        // Request the same recording through visionOS, keeping best audio first.
        // Embedded web was also intermittently refused in the failed-song tests;
        // Android VR exposed a short prefix while refusing later audio ranges.
        command.args(["--extractor-args", "youtube:player_client=visionos"]);
    }
    command
        .args([
            "--ignore-config",
            "--no-plugin-dirs",
            "--no-cache-dir",
            "--no-remote-components",
            "--no-js-runtimes",
            "--js-runtimes",
        ])
        .arg(format!("deno:{}", tools.javascript.display()))
        .args([
            "--no-update",
            "--skip-download",
            "--no-playlist",
            "--dump-single-json",
            "--no-warnings",
            "--no-progress",
            "--no-colors",
            "--socket-timeout",
            "10",
            "--retries",
            "0",
            "--extractor-retries",
            "0",
            "-f",
            if compatibility {
                "bestaudio[protocol=https]/best[protocol=https]"
            } else {
                "bestaudio"
            },
            "--",
        ])
        .arg(format!("https://www.youtube.com/watch?v={id}"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().map_err(|_| {
        "Cannot start the stream resolver. Check the preview runtime files.".to_owned()
    })?;
    #[cfg(windows)]
    let job = match Job::attach(&child) {
        Ok(j) => j,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
    };
    let overflow = Arc::new(AtomicBool::new(false));
    let (o, e) = (child.stdout.take().unwrap(), child.stderr.take().unwrap());
    let flag = overflow.clone();
    let out = thread::spawn(move || read_bounded(o, 4 * 1024 * 1024, flag));
    let flag = overflow.clone();
    let err = thread::spawn(move || read_bounded(e, 64 * 1024, flag));
    let start = Instant::now();
    let mut reason = None;
    let success = loop {
        if generation.load(Ordering::Acquire) != ticket {
            reason = Some("Cancelled");
            break false;
        }
        if start.elapsed() > Duration::from_secs(35) {
            reason = Some("Resolving this recording timed out. Check your connection and retry.");
            break false;
        }
        if overflow.load(Ordering::Acquire) {
            reason = Some("The resolver returned too much data.");
            break false;
        }
        match child.try_wait() {
            Ok(Some(s)) => break s.success(),
            Ok(None) => thread::sleep(Duration::from_millis(30)),
            Err(_) => {
                reason = Some("The stream resolver stopped unexpectedly.");
                break false;
            }
        }
    };
    let _ = child.kill();
    #[cfg(windows)]
    drop(job); // terminates any descendant JavaScript process before joining pipe readers
    let _ = child.wait();
    let raw = out.join().unwrap_or_default();
    let diagnostic = err.join().unwrap_or_default();
    if let Some(s) = reason {
        return Err(s.into());
    }
    if !success {
        let s = String::from_utf8_lossy(&diagnostic);
        return Err(if s.contains("Sign in")||s.contains("not a bot"){ "The provider requires verification for this recording. Try another recording; no browser cookies were accessed." }else if s.contains("unavailable")||s.contains("Private video"){"This recording is unavailable or restricted. Choose another result."}else{"Could not resolve this recording. Check your connection and retry; the provider may have changed."}.into());
    }
    parse(&raw, id)
}
type Slot = Arc<(Mutex<(Option<(u64, usize, String)>, bool)>, Condvar)>;
pub struct Worker {
    slot: Slot,
    generation: Arc<AtomicU64>,
    pub results: Receiver<(u64, usize, Result<Stream, String>)>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    pub fn new() -> Self {
        let slot: Slot = Arc::new((Mutex::new((None, false)), Condvar::new()));
        let generation = Arc::new(AtomicU64::new(0));
        let (tx, results) = mpsc::sync_channel(2);
        let (work, g) = (slot.clone(), generation.clone());
        let thread = thread::spawn(move || {
            let tools = Tools::discover();
            loop {
                let request = {
                    let (lock, wake) = &*work;
                    let mut s = lock.lock().unwrap();
                    while s.0.is_none() && !s.1 {
                        s = wake.wait(s).unwrap();
                    }
                    if s.1 {
                        break;
                    }
                    s.0.take().unwrap()
                };
                let (ticket, id, video) = request;
                let result = resolve(&tools, &video, &g, ticket);
                if g.load(Ordering::Acquire) == ticket {
                    let _ = tx.try_send((ticket, id, result));
                }
            }
        });
        Self {
            slot,
            generation,
            results,
            thread: Some(thread),
        }
    }
    pub fn start(&self, id: usize, video: String) -> u64 {
        let ticket = self.cancel();
        let (lock, wake) = &*self.slot;
        lock.lock().unwrap().0 = Some((ticket, id, video));
        wake.notify_one();
        ticket
    }
    pub fn cancel(&self) -> u64 {
        let n = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        self.slot.0.lock().unwrap().0 = None;
        n
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancel();
        self.slot.0.lock().unwrap().1 = true;
        self.slot.1.notify_one();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stream_probe_checks_range_refusal_and_empty_body() {
        use std::io::{Read, Write};
        for (status, body, expected) in [
            ("206 Partial Content", "x", Ok(())),
            ("200 OK", "x", Ok(())),
            ("403 Forbidden", "no", Err(ProbeError::Refused)),
            ("410 Gone", "no", Err(ProbeError::Refused)),
            (
                "200 OK",
                "",
                Err(ProbeError::Other(
                    "The audio server returned no audio data. Retry or choose another recording."
                        .into(),
                )),
            ),
            (
                "429 Too Many Requests",
                "no",
                Err(ProbeError::Other(
                    "The audio server returned HTTP 429. Retry later or choose another recording."
                        .into(),
                )),
            ),
        ] {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let server = thread::spawn(move || {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut raw = Vec::new();
                while !raw.ends_with(b"\r\n\r\n") {
                    let mut byte = [0];
                    if socket.read(&mut byte).unwrap() == 0 {
                        break;
                    }
                    raw.push(byte[0]);
                    assert!(raw.len() < 8192);
                }
                let request = String::from_utf8(raw).unwrap().to_ascii_lowercase();
                assert!(request.starts_with("get /audio "));
                assert!(request.contains("range: bytes=0-16383\r\n"));
                assert!(request.contains("user-agent: fixture\r\n"));
                write!(
                    socket,
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            });
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let client = reqwest::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap();
            let stream = Stream {
                url: format!("http://{address}/audio"),
                headers: vec![("User-Agent".into(), "fixture".into())],
            };
            assert_eq!(runtime.block_on(verify_stream(&client, &stream)), expected);
            server.join().unwrap();
        }
    }
    #[test]
    fn stream_redirect_hosts_stay_on_https_audio_cdn() {
        for (url, allowed) in [
            ("https://r1.googlevideo.com/audio", true),
            ("https://r1.googlevideo.com:443/audio", true),
            ("http://r1.googlevideo.com/audio", false),
            ("https://r1.googlevideo.com:8443/audio", false),
            ("https://user@r1.googlevideo.com/audio", false),
            ("https://r1.googlevideo.com.evil.example/audio", false),
            ("https://localhost/audio", false),
        ] {
            assert_eq!(audio_host(&reqwest::Url::parse(url).unwrap()), allowed);
        }
    }
    #[test]
    fn prefix_only_stream_is_rejected_when_later_audio_is_forbidden() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for start in [0, 1_048_576] {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut raw = Vec::new();
                while !raw.ends_with(b"\r\n\r\n") {
                    let mut byte = [0];
                    assert_eq!(socket.read(&mut byte).unwrap(), 1);
                    raw.push(byte[0]);
                    assert!(raw.len() < 8192);
                }
                let request = String::from_utf8(raw).unwrap().to_ascii_lowercase();
                assert!(request.contains(&format!("range: bytes={start}-{}\r\n", start + 16_383)));
                let response = if start == 0 {
                    "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes 0-0/2000000\r\nContent-Length: 1\r\nConnection: close\r\n\r\nx"
                } else {
                    "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                };
                socket.write_all(response.as_bytes()).unwrap();
            }
        });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();
        let stream = Stream {
            url: format!("http://{address}/audio"),
            headers: vec![],
        };
        assert_eq!(
            runtime.block_on(verify_stream(&client, &stream)),
            Err(ProbeError::Refused)
        );
        server.join().unwrap();
    }
    #[test]
    fn stream_identity_host_and_header_validation() {
        let id = "abcdefghijk";
        let good = serde_json::json!({"id":id,"url":"https://r1.googlevideo.com/videoplayback?expire=1","acodec":"opus","http_headers":{"User-Agent":"Test"}});
        assert!(parse(&serde_json::to_vec(&good).unwrap(), id).is_ok());
        let mut v = good.clone();
        v["id"] = "other-id".into();
        assert!(parse(&serde_json::to_vec(&v).unwrap(), id).is_err());
        let mut v = good.clone();
        v["url"] = "https://evil.example/audio".into();
        assert!(parse(&serde_json::to_vec(&v).unwrap(), id).is_err());
        let mut v = good;
        v["http_headers"]["User-Agent"] = "x\r\nCookie: secret".into();
        assert!(parse(&serde_json::to_vec(&v).unwrap(), id).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn closing_job_terminates_resolver_and_descendants() {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::{
            Foundation::{CloseHandle, WAIT_OBJECT_0},
            System::{JobObjects::*, Threading::*},
        };
        #[repr(C)]
        struct Ids {
            assigned: u32,
            count: u32,
            ids: [usize; 4],
        }
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "resolver::tests::child_tree_fixture",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x08000000)
            .spawn()
            .unwrap();
        let job = Job::attach(&child).unwrap();
        let start = Instant::now();
        let mut ids = Ids {
            assigned: 0,
            count: 0,
            ids: [0; 4],
        };
        loop {
            unsafe {
                QueryInformationJobObject(
                    job.0,
                    JobObjectBasicProcessIdList,
                    (&mut ids as *mut Ids).cast(),
                    std::mem::size_of_val(&ids) as u32,
                    std::ptr::null_mut(),
                );
            }
            if ids.count >= 2 {
                break;
            }
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "fixture did not create its descendant"
            );
            thread::sleep(Duration::from_millis(10));
        }
        let handles = ids.ids[..ids.count as usize]
            .iter()
            .map(|id| unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, *id as u32) })
            .collect::<Vec<_>>();
        assert!(handles.iter().all(|h| !h.is_null()));
        drop(job);
        // Closing a kill-on-close job can terminate members with exit code zero;
        // signalled process handles, rather than the code, establish termination.
        let _ = child.wait().unwrap();
        for h in handles {
            unsafe {
                assert_eq!(WaitForSingleObject(h, 2000), WAIT_OBJECT_0);
                CloseHandle(h);
            }
        }
    }
    // Ignored fixture entry points are launched only by the job teardown test above.
    #[cfg(windows)]
    #[test]
    #[ignore]
    fn child_tree_fixture() {
        thread::sleep(Duration::from_millis(200));
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "resolver::tests::leaf_fixture"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let _ = child.wait();
    }
    #[cfg(windows)]
    #[test]
    #[ignore]
    fn leaf_fixture() {
        thread::sleep(Duration::from_secs(30));
    }
}
