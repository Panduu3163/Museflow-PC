use crate::{
    AppWindow, TrackRow,
    catalog::{Track, filter_tracks},
    engine::{Command, Engine},
    library::{self, Snapshot},
};
use slint::{ComponentHandle, Model, ModelRc, ModelTracker};
use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::SyncSender,
    },
    thread,
    time::Instant,
};

struct LocalModel {
    tracks: Arc<Vec<Track>>,
    rows: Vec<usize>,
    likes: BTreeSet<usize>,
}
impl Model for LocalModel {
    type Data = TrackRow;
    fn row_count(&self) -> usize {
        self.rows.len()
    }
    fn row_data(&self, row: usize) -> Option<TrackRow> {
        let t = self.tracks.get(*self.rows.get(row)?)?;
        Some(TrackRow {
            id: t.id as i32,
            title: t.title.as_str().into(),
            artist: t.artist.as_str().into(),
            album: t.album.as_str().into(),
            duration: t.duration.as_str().into(),
            cover: -1,
            liked: self.likes.contains(&t.id),
        })
    }
    fn model_tracker(&self) -> &dyn ModelTracker {
        &()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
pub(crate) fn track_model(
    tracks: Arc<Vec<Track>>,
    rows: Vec<usize>,
    likes: BTreeSet<usize>,
) -> ModelRc<TrackRow> {
    ModelRc::new(LocalModel {
        tracks,
        rows,
        likes,
    })
}
struct SearchRequest {
    snapshot: Snapshot,
    page: i32,
    text: String,
    playlist: Option<i64>,
    generation: u64,
}
#[derive(Default)]
struct SearchSlot {
    pending: Option<SearchRequest>,
    stopping: bool,
}
struct Search {
    slot: Arc<(Mutex<SearchSlot>, Condvar)>,
    generation: Arc<AtomicU64>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Search {
    fn new(ui: slint::Weak<AppWindow>) -> Self {
        let slot = Arc::new((Mutex::new(SearchSlot::default()), Condvar::new()));
        let generation = Arc::new(AtomicU64::new(0));
        let (pending, latest) = (slot.clone(), generation.clone());
        let thread = thread::spawn(move || {
            loop {
                let request = {
                    let (lock, wake) = &*pending;
                    let mut slot = lock.lock().unwrap();
                    while slot.pending.is_none() && !slot.stopping {
                        slot = wake.wait(slot).unwrap();
                    }
                    if slot.stopping {
                        break;
                    }
                    slot.pending.take().unwrap()
                };
                let s = request.snapshot;
                let Some(mut rows) =
                    filter_tracks(&s.tracks, request.page, &request.text, &s.likes, || {
                        latest.load(Ordering::Relaxed) != request.generation
                    })
                else {
                    continue;
                };
                if let Some(playlist) = request.playlist {
                    let matching = rows
                        .into_iter()
                        .map(|n| (s.tracks[n].id, n))
                        .collect::<HashMap<_, _>>();
                    rows = s
                        .members
                        .iter()
                        .filter(|(p, _)| *p == playlist)
                        .filter_map(|(_, id)| matching.get(id).copied())
                        .collect();
                }
                let latest = latest.clone();
                let _ = ui.upgrade_in_event_loop(move |ui| {
                    if latest.load(Ordering::Relaxed) != request.generation {
                        return;
                    }
                    ui.set_result_label(format!("{} tracks", rows.len()).into());
                    ui.set_tracks(ModelRc::new(LocalModel {
                        tracks: s.tracks,
                        rows,
                        likes: s.likes,
                    }));
                    ui.set_searching(false);
                });
            }
        });
        Self {
            slot,
            generation,
            thread: Some(thread),
        }
    }
    fn refresh(&self, ui: &AppWindow, snapshot: &Snapshot) {
        if ui.get_online_active() {
            self.generation.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let index = ui.get_collection_index();
        let playlist = (ui.get_page() == 2 && index > 0)
            .then(|| snapshot.playlists.get(index as usize - 1).map(|p| p.0))
            .flatten();
        let request = SearchRequest {
            snapshot: snapshot.clone(),
            page: ui.get_page(),
            text: ui.get_query().into(),
            playlist,
            generation: self.generation.fetch_add(1, Ordering::Relaxed) + 1,
        };
        ui.set_searching(true);
        let (lock, wake) = &*self.slot;
        lock.lock().unwrap().pending = Some(request);
        wake.notify_one();
    }
}
impl Drop for Search {
    fn drop(&mut self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
        let (lock, wake) = &*self.slot;
        lock.lock().unwrap().stopping = true;
        wake.notify_one();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn send(ui: &AppWindow, tx: &SyncSender<Command>, command: Command) {
    if let Err(error) = tx.try_send(command) {
        ui.set_message(
            format!("The action could not be processed: {error}. Please try again.").into(),
        );
    }
}
fn argument(name: &str) -> Option<PathBuf> {
    let args = std::env::args().collect::<Vec<_>>();
    args.iter()
        .position(|a| a == name)
        .and_then(|n| args.get(n + 1))
        .map(PathBuf::from)
}
fn data_dir() -> Result<PathBuf, String> {
    if let Some(path) = argument("--data-dir") {
        return Ok(path);
    }
    if let Some(path) = std::env::var_os("MUSEFLOW_DATA_DIR") {
        return Ok(path.into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    if let Ok(path) = std::fs::read_to_string(exe.with_file_name("portable-data-dir.txt")) {
        let path = PathBuf::from(path.trim());
        return Ok(if path.is_absolute() {
            path
        } else {
            exe.parent().unwrap().join(path)
        });
    }
    std::env::var_os("LOCALAPPDATA")
        .map(|p| PathBuf::from(p).join("MuseFlowPC"))
        .ok_or_else(|| "Set MUSEFLOW_DATA_DIR or pass --data-dir".into())
}
pub(crate) fn runtime_path() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("MUSEFLOW_MPV_LIBRARY") {
        return Ok(path.into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    #[cfg(windows)]
    let name = "libmpv-2.dll";
    #[cfg(not(windows))]
    let name = "libmpv.so.2";
    Ok(exe.with_file_name(name))
}
const EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "m4a", "ogg", "opus", "oga", "aiff", "aif", "wv",
];
fn discover(paths: Vec<PathBuf>, cancel: &AtomicBool) -> (Vec<PathBuf>, usize) {
    let mut pending = paths;
    let mut files = Vec::new();
    let mut errors = 0;
    while let Some(path) = pending.pop() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        if files.len() >= 100_000 {
            errors += pending.len() + 1;
            break;
        }
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            errors += 1;
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            match std::fs::read_dir(&path) {
                Ok(entries) => {
                    for entry in entries {
                        if pending.len() + files.len() >= 100_000 {
                            errors += 1;
                            break;
                        }
                        match entry {
                            Ok(entry) => pending.push(entry.path()),
                            Err(_) => errors += 1,
                        }
                    }
                }
                Err(_) => errors += 1,
            }
        } else if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| EXTENSIONS.contains(&s.to_ascii_lowercase().as_str()))
        {
            files.push(path);
        }
    }
    files.sort();
    (files, errors)
}
fn scan_paths(
    paths: Vec<PathBuf>,
    tx: SyncSender<Command>,
    ui: slint::Weak<AppWindow>,
    cancel: Arc<AtomicBool>,
) {
    let (files, mut failed) = discover(paths, &cancel);
    let total = files.len();
    let mut batch = Vec::new();
    for (n, path) in files.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        match library::scan(&path) {
            Ok(item) => batch.push(item),
            Err(_) => failed += 1,
        }
        if batch.len() >= 500
            && tx
                .send(Command::Import(std::mem::take(&mut batch)))
                .is_err()
        {
            break;
        }
        if n % 100 == 0 {
            let _ = ui.upgrade_in_event_loop(move |ui| {
                ui.set_message(format!("Reading music files: {} / {total}", n + 1).into())
            });
        }
    }
    if !batch.is_empty() {
        let _ = tx.send(Command::Import(batch));
    }
    let _ = tx.send(Command::ImportComplete(failed));
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let data = data_dir()?;
    let runtime = runtime_path()?;
    let null_audio = std::env::args().any(|s| s == "--null-audio");
    let ui = AppWindow::new()?;
    // The Slint preferred size honors the backend/window scale at creation.
    ui.set_demo(false);
    ui.set_selected_title("Your library starts here".into());
    ui.set_selected_artist("Add local music to begin".into());
    ui.set_selected_cover(-1);
    ui.set_data_location(data.display().to_string().into());
    let shared = Arc::new(Mutex::new(Snapshot::default()));
    let state = Rc::new(RefCell::new(Snapshot::default()));
    let initialized = Rc::new(std::cell::Cell::new(false));
    let search = Rc::new(Search::new(ui.as_weak()));
    let online = Rc::new(crate::online_ui::Search::new(ui.as_weak()));
    let artwork_root = data.join("artwork");
    let engine = Engine::spawn(data, runtime, null_audio, ui.as_weak(), shared.clone());
    if let Some(path) = argument("--ready-file") {
        let weak = ui.as_weak();
        let mut done = false;
        ui.window().set_rendering_notifier(move |state, _| {
            if !done
                && matches!(state, slint::RenderingState::AfterRendering)
                && weak.upgrade().is_some_and(|ui| ui.get_library_ready())
            {
                done = true;
                let _ = std::fs::write(
                    &path,
                    format!(
                        "{{\"first_render_from_main_ms\":{}}}\n",
                        started.elapsed().as_millis()
                    ),
                );
            }
        })?;
    }
    let tx = engine.sender.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let (weak, state, shared, search, initialized) = (
            ui.as_weak(),
            state.clone(),
            shared,
            search.clone(),
            initialized,
        );
        ui.on_library_updated(move || {
            if let Some(ui) = weak.upgrade() {
                let snapshot = shared.lock().unwrap().clone();
                ui.set_playlist_names(
                    Rc::new(slint::VecModel::from(
                        std::iter::once("All music".into())
                            .chain(snapshot.playlists.iter().map(|(_, s)| s.as_str().into()))
                            .collect::<Vec<slint::SharedString>>(),
                    ))
                    .into(),
                );
                if ui.get_collection_index() as usize > snapshot.playlists.len() {
                    ui.set_collection_index(0);
                }
                if !initialized.replace(true) {
                    ui.set_dark(snapshot.dark);
                    ui.set_close_to_tray(snapshot.close_to_tray);
                    ui.set_message("".into());
                }
                *state.borrow_mut() = snapshot;
                search.refresh(&ui, &state.borrow());
            }
        });
    }
    {
        let (weak, state, search, online, tx) = (
            ui.as_weak(),
            state.clone(),
            search.clone(),
            online.clone(),
            tx.clone(),
        );
        ui.on_navigate(move |page| {
            if let Some(ui) = weak.upgrade() {
                ui.set_page(page);
                if page == 4 {
                    send(&ui, &tx, Command::RefreshOutputs);
                }
                ui.set_query("".into());
                if ui.get_online_active() {
                    online.reset(&ui);
                } else {
                    online.cancel();
                }
                search.refresh(&ui, &state.borrow());
            }
        });
    }
    {
        let (weak, state, search, online) =
            (ui.as_weak(), state.clone(), search.clone(), online.clone());
        ui.on_search_edited(move |text| {
            if let Some(ui) = weak.upgrade() {
                ui.set_query(text);
                if ui.get_page() == 0 {
                    ui.set_online_selected(false);
                    ui.set_page(1);
                }
                if ui.get_online_active() {
                    online.reset(&ui);
                } else {
                    online.cancel();
                }
                search.refresh(&ui, &state.borrow());
            }
        });
    }
    {
        let (weak, state, search) = (ui.as_weak(), state.clone(), search.clone());
        ui.on_collection_chosen(move |index| {
            if let Some(ui) = weak.upgrade() {
                ui.set_collection_index(index);
                search.refresh(&ui, &state.borrow());
            }
        });
    }
    {
        let (weak, tx, online) = (ui.as_weak(), tx.clone(), online.clone());
        ui.on_select_track(move |id| {
            if let Some(ui) = weak.upgrade() {
                if id <= -2 {
                    if let Some(song) = online.song(id) {
                        send(
                            &ui,
                            &tx,
                            Command::PlayOnline(song.video_id, online.page.borrow().songs.clone()),
                        );
                    }
                    return;
                }
                let model = ui.get_tracks();
                let ids = if let Some(local) = model.as_any().downcast_ref::<LocalModel>() {
                    local.rows.iter().map(|n| local.tracks[*n].id).collect()
                } else {
                    Vec::new()
                };
                send(&ui, &tx, Command::Play(id as usize, ids));
            }
        });
    }
    {
        let (weak, tx) = (ui.as_weak(), tx.clone());
        ui.on_play_queued(move |id| {
            if let Some(ui) = weak.upgrade() {
                send(&ui, &tx, Command::PlayQueued(id as usize));
            }
        });
    }
    macro_rules! simple {
        ($method:ident,$command:expr) => {{
            let (weak, tx) = (ui.as_weak(), tx.clone());
            ui.$method(move || {
                if let Some(ui) = weak.upgrade() {
                    send(&ui, &tx, $command);
                }
            });
        }};
    }
    simple!(on_toggle_play, Command::Toggle);
    simple!(on_next_track, Command::Next);
    simple!(on_previous_track, Command::Previous);
    simple!(on_toggle_shuffle, Command::Shuffle);
    simple!(on_cycle_repeat, Command::Repeat);
    simple!(on_refresh_audio_outputs, Command::RefreshOutputs);
    {
        let (weak, tx) = (ui.as_weak(), tx.clone());
        ui.on_choose_audio_output(move |id| {
            if let Some(ui) = weak.upgrade() {
                send(&ui, &tx, Command::Output(id.to_string()));
            }
        });
    }
    {
        let (weak, tx) = (ui.as_weak(), tx.clone());
        ui.on_seek(move |n| {
            if let Some(ui) = weak.upgrade() {
                send(&ui, &tx, Command::Seek(n as f64));
            }
        });
    }
    {
        let (weak, tx) = (ui.as_weak(), tx.clone());
        ui.on_change_volume(move |n| {
            if let Some(ui) = weak.upgrade() {
                send(&ui, &tx, Command::Volume(n as f64));
            }
        });
    }
    {
        let (weak, tx, online) = (ui.as_weak(), tx.clone(), online.clone());
        ui.on_toggle_like(move |id| {
            if let Some(ui) = weak.upgrade() {
                if id <= -2 {
                    if let Some(song) = online.song(id) {
                        send(&ui, &tx, Command::SaveOnline(song));
                    }
                } else {
                    send(&ui, &tx, Command::Like(id as usize));
                }
            }
        });
    }
    {
        let (weak, tx) = (ui.as_weak(), tx.clone());
        ui.on_theme_changed(move |dark| {
            if let Some(ui) = weak.upgrade() {
                send(&ui, &tx, Command::Theme(dark));
            }
        });
    }
    {
        let (weak, tx) = (ui.as_weak(), tx.clone());
        ui.on_create_playlist(move |name| {
            if let Some(ui) = weak.upgrade() {
                send(&ui, &tx, Command::CreatePlaylist(name.into()));
            }
        });
    }
    {
        let (weak, tx, state) = (ui.as_weak(), tx.clone(), state.clone());
        ui.on_add_selected(move || {
            if let Some(ui) = weak.upgrade()
                && ui.get_collection_index() > 0
                && ui.get_selected_id() >= 0
                && let Some((p, _)) = state
                    .borrow()
                    .playlists
                    .get(ui.get_collection_index() as usize - 1)
            {
                send(
                    &ui,
                    &tx,
                    Command::AddToPlaylist(*p, ui.get_selected_id() as usize),
                );
            }
        });
    }
    for folder in [false, true] {
        let (weak, tx, cancel) = (ui.as_weak(), tx.clone(), cancel.clone());
        let callback = move || {
            if let Some(ui) = weak.upgrade() {
                if ui.get_import_busy() {
                    return;
                }
                ui.set_import_busy(true);
                let (weak, tx, cancel) = (ui.as_weak(), tx.clone(), cancel.clone());
                thread::spawn(move || {
                    let paths = if folder {
                        rfd::FileDialog::new()
                            .set_title("Add a music folder")
                            .pick_folder()
                            .map(|p| vec![p])
                    } else {
                        rfd::FileDialog::new()
                            .set_title("Add music files")
                            .add_filter("Music", EXTENSIONS)
                            .pick_files()
                    };
                    if let Some(paths) = paths {
                        scan_paths(paths, tx, weak, cancel);
                    } else {
                        let _ = weak.upgrade_in_event_loop(|ui| ui.set_import_busy(false));
                    }
                });
            }
        };
        if folder {
            ui.on_import_folder(callback);
        } else {
            ui.on_import_files(callback);
        }
    }
    if let Some(path) = argument("--import") {
        ui.set_import_busy(true);
        let (weak, tx, cancel) = (ui.as_weak(), tx.clone(), cancel.clone());
        thread::spawn(move || scan_paths(vec![path], tx, weak, cancel));
    }
    {
        let (weak, online) = (ui.as_weak(), online.clone());
        ui.on_online_submit(move |more| {
            if let Some(ui) = weak.upgrade() {
                online.search(&ui, more);
            }
        });
    }
    {
        let (weak, online) = (ui.as_weak(), online.clone());
        ui.on_online_cancel(move || {
            if let Some(ui) = weak.upgrade() {
                online.reset(&ui);
                ui.set_online_message("Search cancelled".into());
            }
        });
    }
    {
        let (weak, online, search, state) =
            (ui.as_weak(), online.clone(), search.clone(), state.clone());
        ui.on_search_source(move |remote| {
            if let Some(ui) = weak.upgrade() {
                ui.set_online_selected(remote);
                if remote {
                    online.reset(&ui);
                } else {
                    online.cancel();
                }
                search.refresh(&ui, &state.borrow());
            }
        });
    }
    let _online_checks = crate::online_checks::install(&ui, state.clone())?;
    let _checks = crate::local_checks::install(&ui, state.clone())?;
    let _audio_checks = crate::audio_checks::install(&ui, state.clone())?;
    let _artwork_checks = crate::artwork_checks::install(&ui, state.clone())?;
    let _ui_checks = crate::ui_checks::install(&ui)?;
    let lyrics = crate::lyrics_ui::Lyrics::new(&ui, state.clone());
    let artwork = crate::artwork_ui::Artwork::new(&ui, state.clone(), artwork_root);
    {
        let (weak, tx) = (ui.as_weak(), tx.clone());
        ui.on_close_preference_changed(move |enabled| {
            if let Some(ui) = weak.upgrade() {
                send(&ui, &tx, Command::CloseToTray(enabled));
            }
        });
    }
    {
        let weak = ui.as_weak();
        ui.window().on_close_requested(move || {
            if let Some(ui) = weak.upgrade() {
                if crate::tray::should_hide(ui.get_close_to_tray(), ui.get_tray_ready()) {
                    ui.invoke_tray_action("hide".into());
                } else {
                    let _ = slint::quit_event_loop();
                }
            }
            slint::CloseRequestResponse::KeepWindowShown
        });
    }
    let tray = crate::tray::Tray::new(&ui, tx.clone());
    let _tray_checks = crate::tray_checks::install(&ui)?;
    let _desktop_checks = crate::desktop_checks::install(&ui, state.clone())?;
    ui.show()?;
    let desktop = match crate::desktop::Desktop::new(&ui, tx.clone()) {
        Ok(d) => Some(d),
        Err(e) => {
            ui.set_desktop_status(format!("Windows media controls unavailable: {e}").into());
            None
        }
    };
    slint::run_event_loop_until_quit()?;
    drop(tray);
    drop(desktop);
    drop(artwork);
    drop(lyrics);
    cancel.store(true, Ordering::Relaxed);
    ui.hide()?;
    drop(ui);
    drop(search);
    drop(online);
    drop(engine);
    Ok(())
}
