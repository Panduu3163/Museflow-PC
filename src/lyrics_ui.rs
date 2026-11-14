use crate::{
    AppWindow, LyricRow,
    catalog::Track,
    library::Snapshot,
    lyrics::{Client, Document},
};
use slint::{ComponentHandle, ModelRc};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};
type Slot = Arc<(Mutex<(Option<(u64, Track, bool)>, bool)>, Condvar)>;
pub struct Lyrics {
    _timer: slint::Timer,
    slot: Slot,
    generation: Arc<AtomicU64>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Lyrics {
    pub fn new(ui: &AppWindow, state: Rc<RefCell<Snapshot>>) -> Self {
        let slot: Slot = Arc::new((Mutex::new((None, false)), Condvar::new()));
        let generation = Arc::new(AtomicU64::new(0));
        let (tx, rx) = std::sync::mpsc::sync_channel(2);
        let (s, g) = (slot.clone(), generation.clone());
        let thread = thread::spawn(move || {
            let mut client = Client::new();
            loop {
                let (ticket, track, force) = {
                    let mut lock = s.0.lock().unwrap();
                    while lock.0.is_none() && !lock.1 {
                        lock = s.1.wait(lock).unwrap();
                    }
                    if lock.1 {
                        break;
                    }
                    lock.0.take().unwrap()
                };
                let result = client.as_mut().map_err(|e| e.clone()).and_then(|c| {
                    if force {
                        c.forget(&track);
                    }
                    c.lookup(&track, g.clone(), ticket)
                });
                if g.load(Ordering::Acquire) == ticket {
                    let _ = tx.try_send((ticket, result));
                }
            }
        });
        let weak = ui.as_weak();
        let (s, g) = (slot.clone(), generation.clone());
        let timer = slint::Timer::default();
        let mut active = None;
        let mut document = Document::message("");
        let mut last_line = -2;
        let mut request = 0;
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(150),
            move || {
                let Some(ui) = weak.upgrade() else {
                    return;
                };
                let visible = ui.get_panel_open()
                    && !ui.get_tray_hidden()
                    && !ui.window().is_minimized()
                    && ui.get_lyrics_panel()
                    && ui.get_selected_id() >= 0
                    && ui.get_library_ready();
                let id = visible.then_some(ui.get_selected_id());
                if id != active || ui.get_lyrics_retry() {
                    let force = ui.get_lyrics_retry();
                    ui.set_lyrics_retry(false);
                    active = id;
                    let ticket = g.fetch_add(1, Ordering::AcqRel) + 1;
                    request = ticket;
                    s.0.lock().unwrap().0 = None;
                    document = Document::message("");
                    ui.set_lyric_track_id(id.unwrap_or(-1));
                    last_line = -2;
                    ui.set_lyric_lines(ModelRc::default());
                    ui.set_lyric_current(-1);
                    ui.set_lyric_source("".into());
                    if let Some(id) = id {
                        if let Some(track) = state
                            .borrow()
                            .tracks
                            .iter()
                            .find(|t| t.id == id as usize)
                            .cloned()
                        {
                            ui.set_lyric_status("Loading lyrics…".into());
                            ui.set_lyric_busy(true);
                            s.0.lock().unwrap().0 = Some((ticket, track, force));
                            s.1.notify_one();
                        }
                    } else {
                        ui.set_lyric_busy(false);
                        ui.set_lyric_status("Choose a song to view its lyrics.".into());
                    }
                }
                while let Ok((ticket, result)) = rx.try_recv() {
                    if ticket != request || active.is_none() {
                        continue;
                    }
                    ui.set_lyric_busy(false);
                    match result {
                        Ok(doc) => {
                            ui.set_lyric_status(doc.status.as_str().into());
                            ui.set_lyric_source(doc.source.as_str().into());
                            ui.set_lyric_lines(
                                Rc::new(slint::VecModel::from(
                                    doc.lines
                                        .iter()
                                        .map(|l| LyricRow {
                                            text: l.text.as_str().into(),
                                        })
                                        .collect::<Vec<_>>(),
                                ))
                                .into(),
                            );
                            document = doc;
                            last_line = -2;
                        }
                        Err(e) => {
                            ui.set_lyric_status(e.into());
                        }
                    }
                }
                if active.is_some() {
                    let current = document.current(ui.get_position() as f64);
                    if current != last_line {
                        ui.set_lyric_current(current);
                        last_line = current;
                    }
                }
            },
        );
        Self {
            _timer: timer,
            slot,
            generation,
            thread: Some(thread),
        }
    }
}
impl Drop for Lyrics {
    fn drop(&mut self) {
        self._timer.stop();
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.slot.0.lock().unwrap().1 = true;
        self.slot.1.notify_one();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
