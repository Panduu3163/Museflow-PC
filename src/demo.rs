use crate::catalog::{Track, filter_tracks, sample_catalog};
use crate::{AppWindow, TrackRow};
use slint::{ComponentHandle, Model, ModelRc, ModelTracker};
use std::{
    collections::BTreeSet,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

struct TrackModel {
    catalog: Arc<Vec<Track>>,
    indices: Vec<usize>,
    likes: BTreeSet<usize>,
}
impl Model for TrackModel {
    type Data = TrackRow;
    fn row_count(&self) -> usize {
        self.indices.len()
    }
    fn row_data(&self, row: usize) -> Option<TrackRow> {
        let track = self.catalog.get(*self.indices.get(row)?)?;
        Some(TrackRow {
            id: track.id as i32,
            title: track.title.as_str().into(),
            artist: track.artist.as_str().into(),
            album: track.album.as_str().into(),
            duration: track.duration.as_str().into(),
            cover: track.cover as i32,
            liked: self.likes.contains(&track.id),
        })
    }
    fn model_tracker(&self) -> &dyn ModelTracker {
        &()
    }
}

struct Query {
    generation: u64,
    page: i32,
    text: String,
    likes: BTreeSet<usize>,
}
#[derive(Default)]
struct Pending {
    query: Option<Query>,
    stopping: bool,
}
struct SearchWorker {
    slot: Arc<(Mutex<Pending>, Condvar)>,
    generation: Arc<AtomicU64>,
    thread: Option<thread::JoinHandle<()>>,
}
impl SearchWorker {
    fn new(ui: slint::Weak<AppWindow>, catalog: Arc<Vec<Track>>) -> Self {
        let slot = Arc::new((Mutex::new(Pending::default()), Condvar::new()));
        let generation = Arc::new(AtomicU64::new(0));
        let (pending, latest) = (slot.clone(), generation.clone());
        let thread = thread::spawn(move || {
            loop {
                let request = {
                    let (lock, wake) = &*pending;
                    let mut state = lock.lock().unwrap();
                    while state.query.is_none() && !state.stopping {
                        state = wake.wait(state).unwrap();
                    }
                    if state.stopping {
                        break;
                    }
                    state.query.take().unwrap()
                };
                let Some(indices) = filter_tracks(
                    &catalog,
                    request.page,
                    &request.text,
                    &request.likes,
                    || latest.load(Ordering::Relaxed) != request.generation,
                ) else {
                    continue;
                };
                let (catalog, latest) = (catalog.clone(), latest.clone());
                let _ = ui.upgrade_in_event_loop(move |ui| {
                    if latest.load(Ordering::Relaxed) != request.generation {
                        return;
                    }
                    ui.set_result_label(format!("{} sample tracks", indices.len()).into());
                    ui.set_tracks(ModelRc::new(TrackModel {
                        catalog,
                        indices,
                        likes: request.likes,
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
    fn refresh(&self, ui: &AppWindow, likes: &BTreeSet<usize>) {
        let request = Query {
            generation: self.generation.fetch_add(1, Ordering::Relaxed) + 1,
            page: ui.get_page(),
            text: ui.get_query().into(),
            likes: likes.clone(),
        };
        ui.set_searching(true);
        let (lock, wake) = &*self.slot;
        lock.lock().unwrap().query = Some(request);
        wake.notify_one();
    }
}
impl Drop for SearchWorker {
    fn drop(&mut self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
        let (lock, wake) = &*self.slot;
        lock.lock().unwrap().stopping = true;
        wake.notify_one();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub(crate) fn save_snapshot(
    ui: &AppWindow,
    path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let image = ui.window().take_snapshot()?;
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut encoder = png::Encoder::new(file, image.width(), image.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(image.as_bytes())?;
    Ok(())
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let args: Vec<String> = std::env::args().collect();
    let smoke_dir = args
        .iter()
        .position(|s| s == "--smoke-test")
        .map(|i| {
            args.get(i + 1)
                .map(PathBuf::from)
                .ok_or("--smoke-test needs an output directory")
        })
        .transpose()?;
    let ready_file = args
        .iter()
        .position(|s| s == "--ready-file")
        .map(|i| {
            args.get(i + 1)
                .map(PathBuf::from)
                .ok_or("--ready-file needs a path")
        })
        .transpose()?;
    let tracks = Arc::new(sample_catalog(10_000));
    if args.iter().any(|s| s == "--benchmark-filter") {
        let start = Instant::now();
        for _ in 0..100 {
            std::hint::black_box(filter_tracks(
                &tracks,
                1,
                "velvet",
                &BTreeSet::new(),
                || false,
            ));
        }
        println!(
            "100 filters of 10,000 tracks: {:.2} ms",
            start.elapsed().as_secs_f64() * 1000.0
        );
        return Ok(());
    }
    let ui = AppWindow::new()?;
    ui.window().set_size(slint::LogicalSize::new(1240.0, 820.0));
    let constructed_ms = started.elapsed().as_millis();
    if let Some(path) = ready_file {
        let mut recorded = false;
        ui.window().set_rendering_notifier(move |state, _| {
            if !recorded && matches!(state, slint::RenderingState::AfterRendering) {
                recorded = true;
                let report = format!(
                    "{{\"first_render_from_main_ms\":{}}}\n",
                    started.elapsed().as_millis()
                );
                if let Err(error) = std::fs::write(&path, report) {
                    eprintln!("ready file: {error}");
                }
            }
        })?;
    }
    let likes = Rc::new(std::cell::RefCell::new(BTreeSet::from([2, 6, 18])));
    let worker = Rc::new(SearchWorker::new(ui.as_weak(), tracks.clone()));
    {
        let (weak, worker, likes) = (ui.as_weak(), worker.clone(), likes.clone());
        ui.on_navigate(move |page| {
            if let Some(ui) = weak.upgrade() {
                ui.set_page(page);
                ui.set_query("".into());
                worker.refresh(&ui, &likes.borrow());
            }
        });
    }
    {
        let (weak, worker, likes) = (ui.as_weak(), worker.clone(), likes.clone());
        ui.on_search_edited(move |text| {
            if let Some(ui) = weak.upgrade() {
                ui.set_query(text);
                if ui.get_page() == 0 {
                    ui.set_page(1);
                }
                worker.refresh(&ui, &likes.borrow());
            }
        });
    }
    {
        let (weak, tracks) = (ui.as_weak(), tracks.clone());
        ui.on_select_track(move |id| {
            if let (Some(ui), Some(track)) = (weak.upgrade(), tracks.get(id as usize)) {
                ui.set_selected_id(id);
                ui.set_selected_title(track.title.as_str().into());
                ui.set_selected_artist(track.artist.as_str().into());
                ui.set_selected_album(track.album.as_str().into());
                ui.set_selected_duration(track.duration.as_str().into());
                ui.set_selected_cover(track.cover as i32);
            }
        });
    }
    {
        let (weak, worker, likes) = (ui.as_weak(), worker.clone(), likes.clone());
        ui.on_toggle_like(move |id| {
            if id >= 0 && (id as usize) < 10_000 {
                let mut likes = likes.borrow_mut();
                if !likes.remove(&(id as usize)) {
                    likes.insert(id as usize);
                }
                if let Some(ui) = weak.upgrade() {
                    worker.refresh(&ui, &likes);
                }
            }
        });
    }
    worker.refresh(&ui, &likes.borrow());
    let smoke_timer = slint::Timer::default();
    let watchdog = slint::Timer::default();
    if let Some(dir) = smoke_dir {
        watchdog.start(
            slint::TimerMode::SingleShot,
            Duration::from_secs(30),
            || {
                eprintln!("Smoke test exceeded 30 seconds");
                std::process::exit(1);
            },
        );
        std::fs::create_dir_all(&dir)?;
        let weak = ui.as_weak();
        let step = std::cell::Cell::new(0);
        smoke_timer.start(slint::TimerMode::Repeated, Duration::from_millis(450), move || {
            let Some(ui) = weak.upgrade() else { return; };
            if ui.get_searching() { return; }
            let result: Result<(), Box<dyn std::error::Error>> = (|| {
                match step.get() {
                    0 => {
                        assert_eq!(ui.get_tracks().row_count(), 12); save_snapshot(&ui, dir.join("home-dark.png"))?;
                        let p = ui.get_library_button_origin();
                        let position = slint::LogicalPosition::new(p.x + 30.0, p.y + 18.0);
                        use slint::platform::{WindowEvent, PointerEventButton};
                        ui.window().dispatch_event(WindowEvent::PointerMoved { position });
                        ui.window().dispatch_event(WindowEvent::PointerPressed { position, button: PointerEventButton::Left });
                        ui.window().dispatch_event(WindowEvent::PointerReleased { position, button: PointerEventButton::Left });
                        assert_eq!(ui.get_page(), 2, "Native pointer event must activate Library");
                    }
                    1 => { assert_eq!(ui.get_tracks().row_count(), 10_000); ui.invoke_select_track(9999); ui.set_panel_open(true); save_snapshot(&ui, dir.join("library.png"))?; ui.invoke_search_edited("VELVET".into()); }
                    2 => {
                        assert_eq!(ui.get_tracks().row_count(), 1667); ui.invoke_select_track(ui.get_tracks().row_data(1).unwrap().id); assert_eq!(ui.get_selected_id(), 6);
                        use slint::platform::WindowEvent;
                        ui.window().dispatch_event(WindowEvent::KeyPressed { text: slint::platform::Key::Return.into() });
                        ui.window().dispatch_event(WindowEvent::KeyReleased { text: slint::platform::Key::Return.into() });
                        assert!(ui.get_query().is_empty(), "Keyboard activation must reset Library query");
                        ui.invoke_search_edited("no such recording".into());
                    }
                    3 => { assert_eq!(ui.get_tracks().row_count(), 0); save_snapshot(&ui, dir.join("no-results.png"))?; ui.invoke_navigate(3); }
                    4 => { assert_eq!(ui.get_tracks().row_count(), 3); ui.invoke_toggle_like(6); }
                    5 => { assert_eq!(ui.get_tracks().row_count(), 2); ui.set_dark(false); ui.invoke_navigate(0); }
                    6 => { save_snapshot(&ui, dir.join("home-light.png"))?; ui.window().set_size(slint::LogicalSize::new(820.0, 640.0)); ui.invoke_navigate(4); }
                    7 => { save_snapshot(&ui, dir.join("settings-narrow.png"))?;
                        std::fs::write(dir.join("smoke-result.txt"), format!("PASS: pointer/keyboard Library activation, 10000 rows, filter, empty results, stable ID selection, unlike, theme, narrow window.\nWindow constructed in {constructed_ms} ms (not first-frame timing).\n"))?;
                        slint::quit_event_loop()?;
                    }
                    _ => {}
                }
                Ok(())
            })();
            if let Err(error) = result { eprintln!("smoke test: {error}"); std::process::exit(1); }
            step.set(step.get() + 1);
        });
    }
    ui.run()?;
    Ok(())
}
