use crate::{
    AppWindow,
    artwork::{Cache, SIDE},
    catalog::Track,
    library::Snapshot,
};
use slint::ComponentHandle;
use std::{
    cell::Cell,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
};
type Slot = Arc<(Mutex<(Option<(u64, Track, bool)>, bool)>, Condvar)>;
pub struct Artwork {
    slot: Slot,
    generation: Arc<AtomicU64>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Artwork {
    pub fn new(ui: &AppWindow, state: Rc<std::cell::RefCell<Snapshot>>, root: PathBuf) -> Self {
        let slot: Slot = Arc::new((Mutex::new((None, false)), Condvar::new()));
        let generation = Arc::new(AtomicU64::new(0));
        let (s, g, weak) = (slot.clone(), generation.clone(), ui.as_weak());
        let thread = thread::spawn(move || {
            let mut cache = Cache::new(root);
            loop {
                let (ticket, track, network) = {
                    let mut lock = s.0.lock().unwrap();
                    while lock.0.is_none() && !lock.1 {
                        lock = s.1.wait(lock).unwrap();
                    }
                    if lock.1 {
                        break;
                    }
                    lock.0.take().unwrap()
                };
                let pixels = cache.get(&track, network, &g, ticket);
                let id = track.id as i32;
                let check = g.clone();
                if g.load(Ordering::Acquire) != ticket {
                    continue;
                }
                let _ = weak.upgrade_in_event_loop(move |ui| {
                    if check.load(Ordering::Acquire) != ticket || ui.get_selected_id() != id {
                        return;
                    }
                    if let Some(p) = pixels {
                        let buffer =
                            slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
                                &p.rgba, SIDE, SIDE,
                            );
                        ui.set_artwork(slint::Image::from_rgba8(buffer));
                        ui.set_artwork_ready(true);
                        ui.set_artwork_origin(p.origin.into());
                    } else {
                        ui.set_artwork(slint::Image::default());
                        ui.set_artwork_ready(false);
                        ui.set_artwork_origin("".into());
                    }
                    ui.set_artwork_track_id(id);
                });
            }
        });
        let (s, g, weak) = (slot.clone(), generation.clone(), ui.as_weak());
        let active = Cell::new(None);
        ui.on_artwork_requested(move || {
            let Some(ui) = weak.upgrade() else {
                return;
            };
            if ui.get_demo() {
                return;
            }
            let id = ui.get_selected_id();
            let network = ui.get_loading_audio() || ui.get_audio_loaded();
            if active.get() == Some((id, network)) {
                return;
            }
            active.set(Some((id, network)));
            let ticket = g.fetch_add(1, Ordering::AcqRel) + 1;
            ui.set_artwork_ready(false);
            ui.set_artwork(slint::Image::default());
            ui.set_artwork_track_id(-1);
            ui.set_artwork_origin("".into());
            let track = state
                .borrow()
                .tracks
                .iter()
                .find(|t| t.id == id as usize)
                .cloned();
            let mut lock = s.0.lock().unwrap();
            if lock.1 {
                return;
            }
            lock.0 = track.map(|t| (ticket, t, network));
            s.1.notify_one();
        });
        Self {
            slot,
            generation,
            thread: Some(thread),
        }
    }
}
impl Drop for Artwork {
    fn drop(&mut self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        {
            let mut lock = self.slot.0.lock().unwrap();
            lock.1 = true;
            lock.0 = None;
            self.slot.1.notify_one();
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
