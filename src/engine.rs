use crate::{
    AppWindow,
    catalog::time_label,
    library::{Library, Prepared, Snapshot},
    playback::{Event, Mpv},
    queue::Queue,
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};

pub enum Command {
    Import(Vec<Prepared>),
    ImportComplete(usize),
    Play(usize, Vec<usize>),
    PlayOnline(String, Vec<crate::online::Song>),
    SaveOnline(crate::online::Song),
    Toggle,
    SetPlaying(bool),
    Stop,
    SeekBy(f64),
    Next,
    Previous,
    Seek(f64),
    Volume(f64),
    Shuffle,
    Repeat,
    Like(usize),
    Theme(bool),
    CloseToTray(bool),
    RefreshOutputs,
    Output(String),
    CreatePlaylist(String),
    AddToPlaylist(i64, usize),
    PlayQueued(usize),
    Shutdown,
}
pub struct Engine {
    pub sender: SyncSender<Command>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Engine {
    pub fn spawn(
        data: PathBuf,
        runtime: PathBuf,
        null_audio: bool,
        ui: slint::Weak<AppWindow>,
        shared: Arc<Mutex<Snapshot>>,
    ) -> Self {
        let (sender, receiver) = mpsc::sync_channel(64);
        let thread = thread::spawn(move || {
            let result = (|| -> Result<(), String> {
                std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
                let _instance = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(data.join("instance.lock"))
                    .map_err(|e| format!("Cannot open the library lock: {e}"))?;
                _instance
                    .try_lock()
                    .map_err(|e| format!("Library already in use or cannot be locked: {e}"))?;
                let db = Library::open(&data.join("library.sqlite"))
                    .map_err(|e| format!("Cannot open the library: {e}"))?;
                let queue = db.restore_queue().map_err(|e| e.to_string())?;
                let volume = db
                    .setting("volume")
                    .and_then(|s| s.parse::<f64>().ok())
                    .filter(|v| v.is_finite())
                    .unwrap_or(35.0)
                    .clamp(0.0, 100.0);
                let snapshot = db.snapshot().map_err(|e| e.to_string())?;
                let preferred_output = db
                    .setting("audio_output")
                    .filter(|id| crate::audio_outputs::valid_preference(id))
                    .unwrap_or_else(|| "auto".into());
                let mut core = Core {
                    preferred_output,
                    effective_output: "auto".into(),
                    outputs: vec![crate::audio_outputs::automatic()],
                    db,
                    queue,
                    volume,
                    runtime,
                    null_audio,
                    mpv: None,
                    desired_play: false,
                    loaded: false,
                    loading: None,
                    duration: 0.0,
                    codec: String::new(),
                    snapshot,
                    ui: ui.clone(),
                    shared,
                    last_save: Instant::now(),
                    last_post: Instant::now(),
                    queue_dirty: true,
                    persist_dirty: false,
                    track_indices: std::collections::HashMap::new(),
                    import_summary: (0, 0, 0),
                    resolver: crate::resolver::Worker::new(),
                    resolving: None,
                    active_media: None,
                    retry_used: false,
                };
                core.library_changed()?;
                if let Some(id) = core.queue.current()
                    && let Some(t) = core.snapshot.tracks.iter().find(|t| t.id == id)
                {
                    core.duration = t.seconds;
                }
                core.publish();
                core.run(receiver)
            })();
            if let Err(error) = result {
                let _ = ui.upgrade_in_event_loop(move |ui| ui.set_message(error.into()));
            }
        });
        Self {
            sender,
            thread: Some(thread),
        }
    }
}
impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
struct Core {
    preferred_output: String,
    effective_output: String,
    outputs: Vec<crate::audio_outputs::Output>,
    resolver: crate::resolver::Worker,
    resolving: Option<(u64, usize)>,
    active_media: Option<String>,
    retry_used: bool,
    import_summary: (usize, usize, usize),
    db: Library,
    queue: Queue,
    volume: f64,
    runtime: PathBuf,
    null_audio: bool,
    mpv: Option<Mpv>,
    desired_play: bool,
    loaded: bool,
    loading: Option<Instant>,
    duration: f64,
    codec: String,
    snapshot: Snapshot,
    ui: slint::Weak<AppWindow>,
    shared: Arc<Mutex<Snapshot>>,
    last_save: Instant,
    last_post: Instant,
    queue_dirty: bool,
    persist_dirty: bool,
    track_indices: std::collections::HashMap<usize, usize>,
}
impl Core {
    fn refresh_outputs(&mut self) -> Result<(), String> {
        if self.mpv.is_none() {
            self.mpv = Some(Mpv::open(&self.runtime, self.null_audio)?);
        }
        let mpv = self.mpv.as_ref().unwrap();
        self.outputs = crate::audio_outputs::parse(
            &mpv.string("audio-device-list")
                .ok_or("Audio outputs are unavailable; try refreshing again")?,
        )?;
        let effective =
            crate::audio_outputs::effective(&self.outputs, &self.preferred_output).to_owned();
        if !self.null_audio && self.effective_output != effective {
            mpv.command(&["set", "audio-device", &effective])?;
        }
        self.effective_output = effective;
        self.publish_outputs();
        Ok(())
    }
    fn publish_outputs(&self) {
        let mut outputs = self.outputs.clone();
        let missing = self.preferred_output != self.effective_output;
        if missing {
            outputs.push(crate::audio_outputs::Output {
                id: self.preferred_output.clone(),
                label: "Saved output (unavailable)".into(),
            });
        }
        let selected = outputs
            .iter()
            .position(|o| o.id == self.preferred_output)
            .unwrap_or(0) as i32;
        let label = outputs
            .iter()
            .find(|o| o.id == self.effective_output)
            .map(|o| o.label.as_str())
            .unwrap_or("System default");
        let status = if self.null_audio {
            "Silent integration-test output.".into()
        } else if missing {
            "Saved output is unavailable. Using the system default; Refresh after reconnecting."
                .into()
        } else {
            format!("MuseFlow output: {label}. Other apps keep their own output settings.")
        };
        let _ = self.ui.upgrade_in_event_loop(move |ui| {
            let names = outputs
                .iter()
                .map(|o| slint::SharedString::from(o.label.as_str()))
                .collect::<Vec<_>>();
            let ids = outputs
                .iter()
                .map(|o| slint::SharedString::from(o.id.as_str()))
                .collect::<Vec<_>>();
            ui.set_audio_output_names(std::rc::Rc::new(slint::VecModel::from(names)).into());
            ui.set_audio_output_ids(std::rc::Rc::new(slint::VecModel::from(ids)).into());
            ui.set_audio_output_index(selected);
            ui.set_audio_output_status(status.into());
            ui.set_audio_outputs_ready(true);
        });
    }
    fn choose_output(&mut self, id: String) -> Result<(), String> {
        self.refresh_outputs()?;
        if !self.outputs.iter().any(|o| o.id == id) {
            return Err(
                "That audio output is no longer available. Refresh and choose another.".into(),
            );
        }
        let previous = self.effective_output.clone();
        let mpv = self.mpv.as_ref().unwrap();
        if !self.null_audio {
            mpv.command(&["set", "audio-device", &id])?;
        }
        if let Err(error) = self.db.set("audio_output", &id) {
            if !self.null_audio {
                let _ = mpv.command(&["set", "audio-device", &previous]);
            }
            return Err(format!("Could not save the audio output: {error}"));
        }
        self.preferred_output = id.clone();
        self.effective_output = id;
        self.publish_outputs();
        self.message("Audio output updated.".into());
        Ok(())
    }
    fn library_changed(&mut self) -> Result<(), String> {
        self.snapshot = self.db.snapshot().map_err(|e| e.to_string())?;
        self.track_indices = self
            .snapshot
            .tracks
            .iter()
            .enumerate()
            .map(|(n, t)| (t.id, n))
            .collect();
        self.queue_dirty = true;
        *self.shared.lock().unwrap() = self.snapshot.clone();
        let _ = self.ui.upgrade_in_event_loop(|ui| {
            ui.set_library_ready(true);
            ui.invoke_library_updated();
        });
        Ok(())
    }
    fn message(&self, text: String) {
        let _ = self
            .ui
            .upgrade_in_event_loop(move |ui| ui.set_message(text.into()));
    }
    fn save(&mut self) -> Result<(), String> {
        let result = if self.persist_dirty {
            self.db.save_queue(&self.queue, self.volume)
        } else {
            self.db.save_progress(&self.queue, self.volume)
        };
        result.map_err(|e| format!("Could not save the queue: {e}"))?;
        self.persist_dirty = false;
        self.last_save = Instant::now();
        Ok(())
    }
    fn load(&mut self) -> Result<(), String> {
        let id = self.queue.current().ok_or("Choose a track first")?;
        let track = self
            .snapshot
            .tracks
            .iter()
            .find(|t| t.id == id)
            .cloned()
            .ok_or("The track is no longer in the library")?;
        self.resolver.cancel();
        self.resolving = None;
        self.active_media = None;
        self.loading = None;
        self.loaded = false;
        self.retry_used = false;
        if let Some(mpv) = &self.mpv {
            let _ = mpv.flag("pause", true);
            let _ = mpv.command(&["stop"]);
        }
        if let Some(video) = &track.video_id {
            self.duration = track.seconds;
            self.codec.clear();
            self.desired_play = true;
            let ticket = self.resolver.start(id, video.clone());
            self.resolving = Some((ticket, id));
            self.message("Resolving online audio…".into());
            return Ok(());
        }
        let path = track
            .path
            .as_ref()
            .ok_or("This track has no local file")?
            .clone();
        if !path.is_file() {
            self.loaded = false;
            self.desired_play = false;
            return Err(format!(
                "File missing: {}. Choose another track or restore this file.",
                track.title
            ));
        }
        self.loaded = false;
        self.duration = track.seconds;
        self.codec.clear();
        if self.queue.position >= self.duration - 0.15 {
            self.queue.position = 0.0;
        }
        self.refresh_outputs()?;
        let mpv = self.mpv.as_mut().unwrap();
        mpv.set_number("volume", self.volume)?;
        mpv.load(&path)?;
        self.active_media = Some(path.to_string_lossy().into_owned());
        self.loading = Some(Instant::now());
        self.desired_play = true;
        self.message("Loading audio…".into());
        Ok(())
    }
    fn handle(&mut self, command: Command) -> Result<bool, String> {
        match command {
            Command::RefreshOutputs => {
                if let Err(error) = self.refresh_outputs() {
                    let message = error.clone();
                    let _ = self.ui.upgrade_in_event_loop(move |ui| {
                        ui.set_audio_output_status(message.into())
                    });
                    return Err(error);
                }
            }
            Command::Output(id) => {
                if let Err(error) = self.choose_output(id) {
                    self.publish_outputs();
                    return Err(error);
                }
            }
            Command::CloseToTray(enabled) => {
                self.db
                    .set("close_to_tray", if enabled { "true" } else { "false" })
                    .map_err(|e| e.to_string())?;
            }
            Command::SetPlaying(play) => {
                if play && !self.loaded && self.loading.is_none() && self.resolving.is_none() {
                    if self.queue.current().is_some() {
                        self.load()?;
                    }
                } else {
                    self.desired_play = play;
                    if self.loaded {
                        self.mpv.as_ref().unwrap().flag("pause", !play)?;
                    }
                }
                self.save()?;
            }
            Command::Stop => {
                self.stop_with_error();
                self.queue.position = 0.0;
                self.save()?;
                self.message("".into());
            }
            Command::SeekBy(delta) => {
                if delta.is_finite() {
                    self.seek(self.queue.position + delta)?;
                }
            }
            Command::Shutdown => {
                self.save()?;
                return Ok(false);
            }
            Command::Import(items) => {
                let total = items.len();
                let added = self.db.import(&items).map_err(|e| {
                    self.import_summary.2 += total;
                    e.to_string()
                })?;
                self.import_summary.0 += added;
                self.import_summary.1 += total - added;
                self.library_changed()?;
            }
            Command::ImportComplete(unreadable) => {
                let (added, duplicates, failed) = self.import_summary;
                self.import_summary = (0, 0, 0);
                let text = format!(
                    "Added {added} tracks · {duplicates} already in library · {} unreadable or skipped",
                    failed + unreadable
                );
                let _ = self.ui.upgrade_in_event_loop(move |ui| {
                    ui.set_import_busy(false);
                    ui.set_message(text.into());
                });
            }
            Command::PlayOnline(video, songs) => {
                let selected = songs
                    .iter()
                    .position(|s| s.video_id == video)
                    .ok_or("This search result is no longer available")?;
                let ids = self.db.save_online(&songs).map_err(|e| e.to_string())?;
                self.library_changed()?;
                let id = ids[selected];
                self.queue.replace(ids, id);
                self.queue_dirty = true;
                self.persist_dirty = true;
                self.save()?;
                self.load()?;
            }
            Command::SaveOnline(song) => {
                self.db.save_online(&[song]).map_err(|e| e.to_string())?;
                self.library_changed()?;
                self.message("Saved to your library".into());
            }
            Command::Play(id, ids) => {
                self.queue.replace(ids, id);
                self.queue_dirty = true;
                self.persist_dirty = true;
                self.save()?;
                self.load()?;
            }
            Command::PlayQueued(id) => {
                if let Some(n) = self.queue.order.iter().position(|n| *n == id) {
                    self.queue.cursor = n;
                    self.queue.position = 0.0;
                    self.save()?;
                    self.load()?;
                }
            }
            Command::Toggle => {
                if self.loaded || self.loading.is_some() || self.resolving.is_some() {
                    self.desired_play = !self.desired_play;
                    if self.loaded {
                        self.mpv
                            .as_ref()
                            .unwrap()
                            .flag("pause", !self.desired_play)?;
                    }
                } else {
                    self.load()?;
                }
                self.save()?;
            }
            Command::Next => {
                if self.queue.advance(false).is_some() {
                    self.save()?;
                    self.load()?;
                }
            }
            Command::Previous => {
                if self.queue.position > 3.0 {
                    self.seek(0.0)?;
                } else if self.queue.previous().is_some() {
                    self.save()?;
                    self.load()?;
                }
            }
            Command::Seek(position) => {
                self.seek(position)?;
            }
            Command::Volume(v) => {
                if v.is_finite() {
                    self.volume = v.clamp(0.0, 100.0);
                    if let Some(mpv) = &self.mpv {
                        mpv.set_number("volume", self.volume)?;
                    }
                }
            }
            Command::Shuffle => {
                self.queue.set_shuffle(!self.queue.shuffle);
                self.queue_dirty = true;
                self.persist_dirty = true;
                self.save()?;
            }
            Command::Repeat => {
                self.queue.repeat = self.queue.repeat.next();
                self.save()?;
            }
            Command::Like(id) => {
                self.db.toggle_like(id).map_err(|e| e.to_string())?;
                self.library_changed()?;
            }
            Command::Theme(dark) => {
                self.db
                    .set("dark", if dark { "true" } else { "false" })
                    .map_err(|e| e.to_string())?;
            }
            Command::CreatePlaylist(name) => {
                self.db.create_playlist(&name).map_err(|e| e.to_string())?;
                self.library_changed()?;
            }
            Command::AddToPlaylist(playlist, id) => {
                self.db
                    .add_to_playlist(playlist, id)
                    .map_err(|e| e.to_string())?;
                self.library_changed()?;
                self.message("Added to playlist".into());
            }
        }
        Ok(true)
    }
    fn seek(&mut self, position: f64) -> Result<(), String> {
        if !self.loaded {
            return Ok(());
        }
        if position.is_finite() {
            let n = position.clamp(0.0, self.duration.max(0.0));
            self.mpv
                .as_ref()
                .unwrap()
                .command(&["seek", &n.to_string(), "absolute+exact"])?;
            self.queue.position = n;
        }
        Ok(())
    }
    fn run(&mut self, receiver: Receiver<Command>) -> Result<(), String> {
        loop {
            match receiver.recv_timeout(if self.mpv.is_some() || self.resolving.is_some() {
                Duration::from_millis(80)
            } else {
                Duration::from_secs(3600)
            }) {
                Ok(command) => {
                    let playback = matches!(
                        &command,
                        Command::Play(..)
                            | Command::SetPlaying(..)
                            | Command::Stop
                            | Command::SeekBy(..)
                            | Command::PlayOnline(..)
                            | Command::PlayQueued(..)
                            | Command::Toggle
                            | Command::Next
                            | Command::Previous
                            | Command::Seek(..)
                            | Command::Volume(..)
                    );
                    match self.handle(command) {
                        Ok(false) => break,
                        Ok(true) => {}
                        Err(error) => {
                            if playback {
                                self.stop_with_error();
                            }
                            self.message(error);
                        }
                    }
                    self.publish();
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    self.save()?;
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            while let Ok((ticket, id, result)) = self.resolver.results.try_recv() {
                if self.resolving != Some((ticket, id)) || self.queue.current() != Some(id) {
                    continue;
                }
                self.resolving = None;
                let r = (|| -> Result<(), String> {
                    let stream = result?;
                    self.refresh_outputs()?;
                    let mpv = self.mpv.as_mut().unwrap();
                    mpv.set_number("volume", self.volume)?;
                    mpv.load_media(&stream.url, &stream.headers)?;
                    self.active_media = Some(stream.url);
                    self.loading = Some(Instant::now());
                    self.message("Buffering online audio…".into());
                    Ok(())
                })();
                if let Err(error) = r {
                    self.stop_with_error();
                    self.message(error);
                }
                self.publish();
            }
            let events = self.mpv.as_mut().map(|mpv| mpv.poll()).unwrap_or_default();
            for event in events {
                // Stop or a replacement online selection invalidates the previous
                // decoder's late events while the next recording is being resolved.
                if self.active_media.is_none() {
                    continue;
                }
                match event {
                    Event::Loaded => {
                        let actual = self.mpv.as_ref().and_then(|mpv| mpv.string("path"));
                        let expected = self.active_media.as_deref();
                        let same = actual.as_deref().zip(expected).is_some_and(|(a, b)| {
                            if b.starts_with("https://") {
                                return a == b;
                            }
                            let a = a.replace('\\', "/");
                            let b = b.replace('\\', "/");
                            #[cfg(windows)]
                            {
                                a.trim_start_matches("//?/")
                                    .eq_ignore_ascii_case(b.trim_start_matches("//?/"))
                            }
                            #[cfg(not(windows))]
                            {
                                a == b
                            }
                        });
                        if !same {
                            continue;
                        }
                        let mpv = self.mpv.as_ref().unwrap();
                        self.duration = mpv.number("duration").unwrap_or(self.duration).max(0.0);
                        self.codec = mpv
                            .string("audio-codec-name")
                            .unwrap_or_else(|| "Audio".into());
                        self.loaded = true;
                        self.loading = None;
                        let result = (|| {
                            if self.queue.position > 0.0 {
                                self.seek(self.queue.position)?;
                            }
                            self.mpv.as_ref().unwrap().flag("pause", !self.desired_play)
                        })();
                        if let Err(error) = result {
                            self.stop_with_error();
                            self.message(error);
                        } else {
                            self.message("".into());
                        }
                    }
                    Event::Ended => {
                        self.loaded = false;
                        if self.desired_play
                            && self.queue.shuffle
                            && self.queue.repeat == crate::queue::Repeat::All
                            && self.queue.cursor + 1 >= self.queue.order.len()
                        {
                            self.queue_dirty = true;
                            self.persist_dirty = true;
                        }
                        if self.desired_play && self.queue.advance(true).is_some() {
                            if let Err(error) = self.load() {
                                self.desired_play = false;
                                self.loading = None;
                                self.message(error);
                            }
                        } else {
                            self.desired_play = false;
                            self.queue.position = 0.0;
                        }
                        if let Err(error) = self.save() {
                            self.message(error);
                        }
                    }
                    Event::Failed(error) => {
                        let online = self
                            .queue
                            .current()
                            .and_then(|id| self.snapshot.tracks.iter().find(|t| t.id == id))
                            .is_some_and(|t| t.video_id.is_some());
                        if online && !self.retry_used {
                            let was_playing = self.desired_play;
                            if let Err(e) = self.load() {
                                self.stop_with_error();
                                self.message(e);
                            } else {
                                self.retry_used = true;
                                self.desired_play = was_playing;
                                self.message("Refreshing the online stream…".into());
                            }
                            continue;
                        }
                        self.loaded = false;
                        self.loading = None;
                        self.desired_play = false;
                        self.message(if online {"The online stream could not play after retrying. Check your connection or choose another result.".into()}else{format!("Could not play this file: {error}")});
                    }
                }
            }
            if self
                .loading
                .is_some_and(|start| start.elapsed() > Duration::from_secs(15))
            {
                self.loading = None;
                self.loaded = false;
                self.desired_play = false;
                if let Some(mpv) = &self.mpv {
                    let _ = mpv.command(&["stop"]);
                }
                self.message("This file took too long to open. Try another track.".into());
            }
            if self.loaded
                && self.last_post.elapsed() > Duration::from_millis(250)
                && let Some(n) = self.mpv.as_ref().and_then(|mpv| mpv.number("time-pos"))
            {
                self.queue.position = n;
            }
            if self.last_post.elapsed() > Duration::from_millis(250) {
                self.publish();
            }
            if self.last_save.elapsed() > Duration::from_secs(5)
                && self.loaded
                && let Err(error) = self.save()
            {
                self.message(error);
            }
        }
        Ok(())
    }
    fn stop_with_error(&mut self) {
        self.resolver.cancel();
        self.resolving = None;
        self.active_media = None;
        self.loading = None;
        self.loaded = false;
        self.desired_play = false;
        if let Some(mpv) = &self.mpv {
            let _ = mpv.flag("pause", true);
            let _ = mpv.command(&["stop"]);
        }
    }
    fn publish(&mut self) {
        self.last_post = Instant::now();
        let track = self
            .queue
            .current()
            .and_then(|id| self.snapshot.tracks.iter().find(|t| t.id == id))
            .cloned();
        let (position, duration, volume) = (
            self.queue.position as f32,
            self.duration as f32,
            self.volume as f32,
        );
        let (playing, loading, loaded) = (
            self.desired_play,
            self.loading.is_some() || self.resolving.is_some(),
            self.loaded,
        );
        let (next, previous) = (
            self.queue.can_next(),
            self.queue.can_previous() || self.queue.position > 3.0,
        );
        let (shuffle, repeat, codec) = (
            self.queue.shuffle,
            self.queue.repeat.label().to_owned(),
            self.codec.clone(),
        );
        let queue_plain = self.queue_dirty.then(|| {
            (
                self.snapshot.tracks.clone(),
                self.queue
                    .order
                    .iter()
                    .filter_map(|id| self.track_indices.get(id).copied())
                    .collect::<Vec<_>>(),
                self.snapshot.likes.clone(),
            )
        });
        self.queue_dirty = false;
        let _ = self.ui.upgrade_in_event_loop(move |ui| {
            if let Some(t) = track {
                ui.set_selected_id(t.id as i32);
                ui.set_selected_title(t.title.as_str().into());
                ui.set_selected_artist(t.artist.as_str().into());
                ui.set_selected_album(t.album.as_str().into());
                ui.set_selected_duration(time_label(t.seconds).into());
                ui.set_selected_cover(-1);
            }
            ui.set_playing(playing);
            ui.set_loading_audio(loading);
            ui.set_audio_loaded(loaded);
            ui.set_position(position);
            ui.set_duration(duration);
            ui.set_volume(volume);
            ui.set_position_label(time_label(position as f64).into());
            ui.set_duration_label(time_label(duration as f64).into());
            ui.set_has_next(next);
            ui.set_has_previous(previous);
            ui.set_shuffle_enabled(shuffle);
            ui.set_repeat_label(repeat.into());
            ui.set_codec_label(codec.into());
            if let Some((tracks, rows, likes)) = queue_plain {
                ui.set_queue_tracks(crate::local::track_model(tracks, rows, likes));
            }
            ui.invoke_artwork_requested();
            ui.invoke_playback_updated();
        });
    }
}
