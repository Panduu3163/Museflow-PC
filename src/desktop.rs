//! Windows SMTC shares the existing queue/playback owner. Linux integration is later work.
use crate::{AppWindow, engine::Command};
use slint::ComponentHandle;
use std::sync::mpsc::SyncSender;
#[cfg(windows)]
fn action(event: souvlaki::MediaControlEvent) -> Option<Command> {
    use souvlaki::{MediaControlEvent as E, SeekDirection};
    let direction = |d: SeekDirection| {
        if d == SeekDirection::Forward {
            1.0
        } else {
            -1.0
        }
    };
    match event {
        E::Play => Some(Command::SetPlaying(true)),
        E::Pause => Some(Command::SetPlaying(false)),
        E::Toggle => Some(Command::Toggle),
        E::Stop => Some(Command::Stop),
        E::Next => Some(Command::Next),
        E::Previous => Some(Command::Previous),
        E::SetPosition(p) => Some(Command::Seek(p.0.as_secs_f64())),
        E::Seek(d) => Some(Command::SeekBy(direction(d) * 10.0)),
        E::SeekBy(d, t) => Some(Command::SeekBy(direction(d) * t.as_secs_f64())),
        E::SetVolume(v) => Some(Command::Volume(v * 100.0)),
        _ => None,
    }
}

#[cfg(windows)]
struct State {
    controls: Option<souvlaki::MediaControls>,
    attempts: u32,
    closed: bool,
    metadata: Option<(i32, String, String, String, u64)>,
    playback: Option<souvlaki::MediaPlayback>,
}
pub struct Desktop {
    _timer: slint::Timer,
    #[cfg(windows)]
    state: std::rc::Rc<std::cell::RefCell<State>>,
}
#[cfg(windows)]
fn connect(ui: &AppWindow, tx: SyncSender<Command>) -> Result<souvlaki::MediaControls, String> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use souvlaki::{MediaControls, PlatformConfig};
    let handle = ui.window().window_handle();
    let hwnd = match handle.window_handle().map_err(|e| e.to_string())?.as_raw() {
        RawWindowHandle::Win32(h) => h.hwnd.get() as *mut std::ffi::c_void,
        _ => return Err("No native Windows handle for media controls".into()),
    };
    let mut controls = MediaControls::new(PlatformConfig {
        display_name: "MuseFlow PC",
        dbus_name: "museflow_pc",
        hwnd: Some(hwnd),
    })
    .map_err(|e| e.to_string())?;
    controls
        .attach(move |event| {
            if let Some(command) = action(event) {
                let _ = tx.try_send(command);
            }
        })
        .map_err(|e| e.to_string())?;
    Ok(controls)
}
#[cfg(windows)]
fn sync(ui: &AppWindow, state: &mut State, tx: SyncSender<Command>) {
    use souvlaki::{MediaMetadata, MediaPlayback, MediaPosition};
    use std::time::Duration;
    if state.closed || !ui.get_library_ready() {
        return;
    }
    if state.controls.is_none() {
        if state.attempts >= 20 {
            return;
        }
        state.attempts += 1;
        match connect(ui, tx) {
            Ok(c) => state.controls = Some(c),
            Err(e) => {
                ui.set_desktop_status(format!("Windows media controls unavailable: {e}").into());
                return;
            }
        }
    }
    let result = (|| -> Result<(), String> {
        let controls = state.controls.as_mut().unwrap();
        let selected = ui.get_selected_id() >= 0;
        let duration = (ui.get_duration() as f64).max(0.0);
        let metadata = (
            ui.get_selected_id(),
            ui.get_selected_title().to_string(),
            ui.get_selected_artist().to_string(),
            ui.get_selected_album().to_string(),
            duration.round() as u64,
        );
        if selected && state.metadata.as_ref() != Some(&metadata) {
            controls
                .set_metadata(MediaMetadata {
                    title: Some(&metadata.1),
                    artist: Some(&metadata.2),
                    album: Some(&metadata.3),
                    duration: Some(Duration::from_secs_f64(duration)),
                    cover_url: None,
                })
                .map_err(|e| e.to_string())?;
            state.metadata = Some(metadata);
        }
        let progress = Some(MediaPosition(Duration::from_secs_f64(
            (ui.get_position() as f64).max(0.0),
        )));
        let playback = if !selected
            || (!ui.get_audio_loaded() && !ui.get_loading_audio() && !ui.get_playing())
        {
            MediaPlayback::Stopped
        } else if ui.get_playing() && ui.get_audio_loaded() {
            MediaPlayback::Playing { progress }
        } else {
            MediaPlayback::Paused { progress }
        };
        if state.playback.as_ref() != Some(&playback) {
            controls
                .set_playback(playback.clone())
                .map_err(|e| e.to_string())?;
            state.playback = Some(playback);
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            ui.set_desktop_ready(true);
            ui.set_desktop_status("Windows media controls connected".into());
        }
        Err(e) => {
            ui.set_desktop_ready(false);
            ui.set_desktop_status(
                format!("Windows media update failed: {e}. In-app controls remain available.")
                    .into(),
            );
        }
    }
}
impl Desktop {
    #[cfg(windows)]
    pub fn new(ui: &AppWindow, tx: SyncSender<Command>) -> Result<Self, String> {
        use std::{cell::RefCell, rc::Rc, time::Duration};
        let state = Rc::new(RefCell::new(State {
            controls: None,
            attempts: 0,
            closed: false,
            metadata: None,
            playback: None,
        }));
        let (weak, s, sender) = (ui.as_weak(), state.clone(), tx.clone());
        ui.on_playback_updated(move || {
            if let Some(ui) = weak.upgrade() {
                sync(&ui, &mut s.borrow_mut(), sender.clone());
            }
        });
        let (weak, s) = (ui.as_weak(), state.clone());
        let timer = slint::Timer::default();
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(250),
            move || {
                if let Some(ui) = weak.upgrade() {
                    sync(&ui, &mut s.borrow_mut(), tx.clone());
                }
            },
        );
        Ok(Self {
            _timer: timer,
            state,
        })
    }
    #[cfg(not(windows))]
    pub fn new(_ui: &AppWindow, _tx: SyncSender<Command>) -> Result<Self, String> {
        Err("Desktop media controls are currently supported on Windows only.".into())
    }
}
impl Drop for Desktop {
    fn drop(&mut self) {
        self._timer.stop();
        #[cfg(windows)]
        {
            let mut state = self.state.borrow_mut();
            state.closed = true;
            state.controls = None;
        }
    }
}
#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn media_actions_are_explicit_and_seek_direction_is_preserved() {
        use souvlaki::{MediaControlEvent as E, SeekDirection};
        assert!(matches!(action(E::Play), Some(Command::SetPlaying(true))));
        assert!(matches!(action(E::Pause), Some(Command::SetPlaying(false))));
        assert!(matches!(action(E::Stop), Some(Command::Stop)));
        assert!(
            matches!(action(E::Seek(SeekDirection::Backward)),Some(Command::SeekBy(n))if n== -10.0)
        );
        assert!(action(E::OpenUri("https://example.com".into())).is_none());
    }
}
