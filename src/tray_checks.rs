//! Opt-in lifecycle journeys. Sends WM_CLOSE only to this test app's own HWND.
use crate::AppWindow;
use slint::{ComponentHandle, Model};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};
pub struct Checks {
    _timer: slint::Timer,
    _watchdog: slint::Timer,
    out: Option<PathBuf>,
    expected_exit: Rc<Cell<bool>>,
    description: Rc<RefCell<String>>,
}
#[cfg(windows)]
fn close(ui: &AppWindow) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    if let Ok(h) = ui.window().window_handle().window_handle()
        && let RawWindowHandle::Win32(h) = h.as_raw()
    {
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                h.hwnd.get() as _,
                windows_sys::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                0,
                0,
            );
        }
    }
}
#[cfg(not(windows))]
fn close(_ui: &AppWindow) {}
pub fn install(ui: &AppWindow) -> Result<Checks, Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let mode = [
        "tray-lifecycle",
        "tray-exit",
        "tray-restore",
        "tray-fallback",
        "tray-media",
    ]
    .into_iter()
    .find_map(|s| {
        args.iter()
            .position(|v| v == &format!("--{s}-smoke-test"))
            .and_then(|n| args.get(n + 1))
            .map(|p| (s, PathBuf::from(p)))
    });
    let timer = slint::Timer::default();
    let watchdog = slint::Timer::default();
    let expected_exit = Rc::new(Cell::new(false));
    let description = Rc::new(RefCell::new(String::new()));
    let out = mode.as_ref().map(|(_, p)| p.clone());
    if let Some((mode, out)) = mode {
        std::fs::create_dir_all(&out)?;
        let weak = ui.as_weak();
        let phase = Cell::new(0);
        let saved_position = Cell::new(0.0);
        let elapsed = RefCell::new(Instant::now());
        let done = expected_exit.clone();
        let message = description.clone();
        let timeout = ui.as_weak();
        watchdog.start(
            slint::TimerMode::SingleShot,
            Duration::from_secs(35),
            move || {
                if let Some(ui) = timeout.upgrade() {
                    eprintln!(
                        "Tray timeout: {} hidden={} visible={} ready={}",
                        ui.get_tray_status(),
                        ui.get_tray_hidden(),
                        ui.get_native_window_visible(),
                        ui.get_tray_ready()
                    );
                }
                std::process::exit(1);
            },
        );
        timer.start(slint::TimerMode::Repeated,Duration::from_millis(100),move||{
            let Some(ui)=weak.upgrade()else{return;};if !ui.get_library_ready()||ui.get_import_busy()||ui.get_searching(){return;}
            if mode=="tray-media"{
                std::fs::write(out.join("state.json"),serde_json::to_vec(&serde_json::json!({"title":ui.get_selected_title().to_string(),"playing":ui.get_playing(),"loaded":ui.get_audio_loaded(),"position":ui.get_position(),"hidden":ui.get_tray_hidden(),"visible":ui.get_native_window_visible(),"desktop_ready":ui.get_desktop_ready()})).unwrap()).unwrap();
                if phase.get()==0&&ui.get_tray_ready()&&ui.get_tracks().row_count()>=2{ui.invoke_select_track(ui.get_tracks().row_data(0).unwrap().id);ui.invoke_tray_action("hide".into());phase.set(1);}
                if phase.get()==1&&ui.get_playing()&&ui.get_audio_loaded()&&ui.get_tray_hidden()&&!ui.get_native_window_visible()&&ui.get_position()>0.2{std::fs::write(out.join("READY"),ui.get_selected_title().as_str()).unwrap();phase.set(2);}
                if phase.get()==2&&out.join("FINISH").exists(){*message.borrow_mut()="Windows media commands and metadata remain responsive while the native player is hidden in the tray".into();done.set(true);ui.invoke_tray_action("exit".into());phase.set(9);}return;
            }
            if mode=="tray-fallback"{if !ui.get_tray_ready()&&ui.get_tray_status().contains("unavailable"){ui.set_close_to_tray(true);*message.borrow_mut()="A failed tray cannot hide the app; native Close exits despite the saved opt-in".into();done.set(true);close(&ui);}return;}
            if !ui.get_tray_ready(){return;}
            if mode=="tray-restore"{assert!(ui.get_close_to_tray());assert!(!ui.get_playing()&&!ui.get_audio_loaded());*message.borrow_mut()="Close-to-tray preference and paused queue restored".into();done.set(true);slint::quit_event_loop().unwrap();return;}
            match phase.get(){
                0=>{if ui.get_tracks().row_count()<2{return;}ui.invoke_select_track(ui.get_tracks().row_data(0).unwrap().id);ui.set_close_to_tray(true);phase.set(1);}
                1=>{if !ui.get_audio_loaded()||ui.get_position()<0.2{return;}saved_position.set(ui.get_position());*elapsed.borrow_mut()=Instant::now();close(&ui);phase.set(2);}
                2=>{if !ui.get_tray_hidden()||ui.get_native_window_visible()||elapsed.borrow().elapsed()<Duration::from_secs(1){return;}assert!(ui.get_playing()&&ui.get_audio_loaded());assert!(ui.get_position()>saved_position.get()+0.3);if mode=="tray-exit"{*message.borrow_mut()="Exit action terminates the hidden player through normal cleanup".into();done.set(true);ui.invoke_tray_action("exit".into());phase.set(9);}else{ui.invoke_tray_action("show".into());phase.set(3);}}
                3=>{if ui.get_tray_hidden()||!ui.get_native_window_visible(){return;}ui.window().set_minimized(true);phase.set(4);}
                4=>{if !ui.window().is_minimized(){return;}ui.invoke_tray_action("show".into());phase.set(5);}
                5=>{if ui.window().is_minimized()||ui.get_tray_hidden()||!ui.get_native_window_visible(){return;}ui.set_close_to_tray(false);ui.invoke_navigate(4);phase.set(6);}
                6=>{crate::demo::save_snapshot(&ui,out.join("tray-settings.png")).unwrap();*message.borrow_mut()="Native Close hides only after opt-in; decoding continues while hidden; Show restores hidden/minimized window; default Close exits".into();done.set(true);close(&ui);phase.set(9);}
                _=>{}
            }
        });
    }
    if out.is_none()
        && let Some(n) = args.iter().position(|s| s == "--auto-exit-after-ms")
    {
        let ms = args
            .get(n + 1)
            .ok_or("Missing auto-exit timeout")?
            .parse::<u64>()?;
        if !(500..=60000).contains(&ms) {
            return Err("Auto-exit timeout must be 500–60000 ms".into());
        }
        watchdog.start(
            slint::TimerMode::SingleShot,
            Duration::from_millis(ms),
            || {
                let _ = slint::quit_event_loop();
            },
        );
    }
    Ok(Checks {
        _timer: timer,
        _watchdog: watchdog,
        out,
        expected_exit,
        description,
    })
}
impl Drop for Checks {
    fn drop(&mut self) {
        self._timer.stop();
        self._watchdog.stop();
        if self.expected_exit.get()
            && let Some(out) = &self.out
        {
            let _ = std::fs::write(
                out.join("PASS.txt"),
                format!(
                    "{}\nNormal event-loop exit observed. Null output; no acoustic claim.\n",
                    self.description.borrow()
                ),
            );
        }
    }
}
