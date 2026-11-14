use crate::{
    AppWindow, TrackRow,
    online::{Catalog, Page, Song},
};
use slint::ModelRc;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
};
type Slot = Arc<(Mutex<(Option<Request>, bool)>, Condvar)>;
struct Request {
    ticket: u64,
    query: String,
    previous: Page,
    more: bool,
}
pub struct Search {
    slot: Slot,
    generation: Arc<AtomicU64>,
    pub page: Rc<RefCell<Page>>,
    thread: Option<thread::JoinHandle<()>>,
}
fn model(songs: &[Song]) -> ModelRc<TrackRow> {
    Rc::new(slint::VecModel::from(
        songs
            .iter()
            .enumerate()
            .map(|(n, s)| TrackRow {
                id: -2 - n as i32,
                title: s.title.as_str().into(),
                artist: s.artist.as_str().into(),
                album: s.album.as_str().into(),
                duration: crate::catalog::time_label(s.seconds).into(),
                cover: -1,
                liked: false,
            })
            .collect::<Vec<_>>(),
    ))
    .into()
}
impl Search {
    pub fn new(ui: slint::Weak<AppWindow>) -> Self {
        let slot: Slot = Arc::new((Mutex::new((None, false)), Condvar::new()));
        let generation = Arc::new(AtomicU64::new(0));
        let page = Rc::new(RefCell::new(Page::default()));
        // Send plain results to an event-loop callback; Rc state never crosses threads.
        let (tx, rx) = std::sync::mpsc::sync_channel::<(u64, Result<Page, String>)>(2);
        let (work, g) = (slot.clone(), generation.clone());
        let worker_ui = ui.clone();
        let thread = thread::spawn(move || {
            let catalog = Catalog::new();
            loop {
                let r = {
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
                let result = catalog
                    .as_ref()
                    .map_err(Clone::clone)
                    .and_then(|c| {
                        c.search(
                            &r.query,
                            if r.more {
                                r.previous.continuation.as_deref()
                            } else {
                                None
                            },
                            g.clone(),
                            r.ticket,
                        )
                    })
                    .map(|mut next| {
                        if r.more {
                            let mut songs = r.previous.songs;
                            let mut ids = songs
                                .iter()
                                .map(|s| s.video_id.clone())
                                .collect::<std::collections::BTreeSet<_>>();
                            for s in next.songs {
                                if ids.insert(s.video_id.clone()) && songs.len() < 500 {
                                    songs.push(s);
                                }
                            }
                            next.songs = songs;
                        }
                        if next.songs.len() >= 500 {
                            next.continuation = None;
                        }
                        next
                    });
                if g.load(Ordering::Acquire) == r.ticket {
                    let _ = tx.try_send((r.ticket, result));
                    let _ = worker_ui.upgrade_in_event_loop(|ui| ui.invoke_online_updated());
                }
            }
        });
        let (p, g, weak) = (page.clone(), generation.clone(), ui.clone());
        // Registered once, drained only on completed worker tasks (no idle polling timer).
        if let Some(ui) = weak.upgrade() {
            ui.on_online_updated(move || {
                while let Ok((ticket, result)) = rx.try_recv() {
                    if g.load(Ordering::Acquire) != ticket {
                        continue;
                    }
                    if let Some(ui) = weak.upgrade() {
                        if !ui.get_online_active() || ui.get_page() != 1 {
                            continue;
                        }
                        ui.set_searching(false);
                        match result {
                            Ok(next) => {
                                ui.set_tracks(model(&next.songs));
                                ui.set_result_label(
                                    format!(
                                        "{} online {}",
                                        next.songs.len(),
                                        if next.songs.len() == 1 {
                                            "song"
                                        } else {
                                            "songs"
                                        }
                                    )
                                    .into(),
                                );
                                ui.set_online_more(next.continuation.is_some());
                                *p.borrow_mut() = next;
                                ui.set_online_message("".into());
                            }
                            Err(e) => {
                                ui.set_online_message(e.into());
                                ui.set_online_more(p.borrow().continuation.is_some());
                            }
                        }
                    }
                }
            });
        }
        Self {
            slot,
            generation,
            page,
            thread: Some(thread),
        }
    }
    pub fn search(&self, ui: &AppWindow, more: bool) {
        let query = ui.get_query().trim().chars().take(200).collect::<String>();
        if query.is_empty() {
            ui.set_online_message("Enter a song or artist, then choose Search.".into());
            return;
        }
        let previous = self.page.borrow().clone();
        if more && previous.continuation.is_none() {
            return;
        }
        let ticket = self.cancel();
        if !more {
            *self.page.borrow_mut() = Page::default();
            ui.set_tracks(model(&[]));
            ui.set_online_more(false);
        }
        ui.set_searching(true);
        ui.set_online_message(
            if more {
                "Loading more songs…"
            } else {
                "Searching YouTube Music…"
            }
            .into(),
        );
        self.slot.0.lock().unwrap().0 = Some(Request {
            ticket,
            query,
            previous,
            more,
        });
        self.slot.1.notify_one();
    }
    pub fn cancel(&self) -> u64 {
        let n = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        self.slot.0.lock().unwrap().0 = None;
        n
    }
    pub fn reset(&self, ui: &AppWindow) {
        self.cancel();
        *self.page.borrow_mut() = Page::default();
        ui.set_tracks(model(&[]));
        ui.set_online_more(false);
        ui.set_searching(false);
        ui.set_online_message("Search YouTube Music by song or artist. No account needed.".into());
    }
    pub fn song(&self, id: i32) -> Option<Song> {
        id.checked_neg()
            .and_then(|n| n.checked_sub(2))
            .and_then(|n| self.page.borrow().songs.get(n as usize).cloned())
    }
}
impl Drop for Search {
    fn drop(&mut self) {
        self.cancel();
        self.slot.0.lock().unwrap().1 = true;
        self.slot.1.notify_one();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
