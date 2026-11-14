//! Optional Windows tray lifecycle. The native player HWND stays alive while hidden.
use crate::{AppWindow, engine::Command};
use slint::ComponentHandle;
use std::sync::mpsc::{self, SyncSender};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Show,
    Hide,
    Toggle,
    Next,
    Previous,
    Exit,
}
fn action(id: &str) -> Option<Action> {
    match id {
        "museflow.show" | "show" => Some(Action::Show),
        "hide" => Some(Action::Hide),
        "museflow.toggle" => Some(Action::Toggle),
        "museflow.next" => Some(Action::Next),
        "museflow.previous" => Some(Action::Previous),
        "museflow.exit" | "exit" => Some(Action::Exit),
        _ => None,
    }
}
pub fn should_hide(preference: bool, ready: bool) -> bool {
    preference && ready
}
pub struct Tray {
    _timer: slint::Timer,
}
#[cfg(windows)]
mod native {
    use super::*;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use tray_icon::{
        Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
        menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    };
    use windows_sys::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{
            IsWindowVisible, SW_HIDE, SW_RESTORE, SetForegroundWindow, ShowWindow,
        },
    };
    pub struct Native {
        _handle: slint::WindowHandle,
        hwnd: HWND,
        icon: TrayIcon,
        toggle: MenuItem,
        next: MenuItem,
        previous: MenuItem,
        last: Option<(i32, bool, bool, bool, String)>,
    }
    fn icon() -> Result<Icon, String> {
        let mut decoder = png::Decoder::new(std::io::Cursor::new(include_bytes!(
            "../assets/branding/android-launcher-foreground.png"
        )));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder
            .read_info()
            .map_err(|_| "Cannot decode the tray logo")?;
        let size = reader
            .output_buffer_size()
            .filter(|n| *n <= 4 * 1024 * 1024)
            .ok_or("Tray logo is too large")?;
        let mut bytes = vec![0; size];
        let info = reader
            .next_frame(&mut bytes)
            .map_err(|_| "Cannot read the tray logo")?;
        if info.color_type != png::ColorType::Rgba {
            return Err("Tray logo must use RGBA".into());
        }
        let mut rgba = vec![0; 32 * 32 * 4];
        for y in 0..32usize {
            for x in 0..32usize {
                let source = (((y * info.height as usize / 32) * info.width as usize)
                    + (x * info.width as usize / 32))
                    * 4;
                let dest = (y * 32 + x) * 4;
                // Brand-coloured disc protects the white glyph in both Windows themes.
                if (x as f32 - 15.5).hypot(y as f32 - 15.5) > 15.0 {
                    continue;
                }
                let a = bytes[source + 3] as u32;
                for (c, bg) in [42u32, 12, 24].iter().enumerate() {
                    rgba[dest + c] = ((bytes[source + c] as u32 * a + bg * (255 - a)) / 255) as u8;
                }
                rgba[dest + 3] = 255;
            }
        }
        Icon::from_rgba(rgba, 32, 32).map_err(|e| e.to_string())
    }
    impl Native {
        pub fn new(ui: &AppWindow) -> Result<Self, String> {
            let handle = ui.window().window_handle();
            let hwnd = match handle.window_handle().map_err(|e| e.to_string())?.as_raw() {
                RawWindowHandle::Win32(h) => h.hwnd.get() as HWND,
                _ => return Err("No Windows player handle".into()),
            };
            let menu = Menu::new();
            let show = MenuItem::with_id("museflow.show", "Show MuseFlow", true, None);
            let toggle = MenuItem::with_id("museflow.toggle", "Play / Pause", false, None);
            let next = MenuItem::with_id("museflow.next", "Next", false, None);
            let previous = MenuItem::with_id("museflow.previous", "Previous", false, None);
            let exit = MenuItem::with_id("museflow.exit", "Exit MuseFlow", true, None);
            menu.append_items(&[
                &show,
                &toggle,
                &previous,
                &next,
                &PredefinedMenuItem::separator(),
                &exit,
            ])
            .map_err(|e| e.to_string())?;
            let icon = TrayIconBuilder::new()
                .with_id("museflow.tray")
                .with_icon(icon()?)
                .with_tooltip("MuseFlow PC")
                .with_menu(Box::new(menu))
                .with_menu_on_left_click(false)
                .build()
                .map_err(|e| e.to_string())?;
            Ok(Self {
                _handle: handle,
                hwnd,
                icon,
                toggle,
                next,
                previous,
                last: None,
            })
        }
        pub fn show(&self, ui: &AppWindow) {
            unsafe {
                ShowWindow(self.hwnd, SW_RESTORE);
                SetForegroundWindow(self.hwnd);
            }
            ui.set_tray_hidden(false);
            ui.window().request_redraw();
        }
        pub fn hide(&self, ui: &AppWindow) {
            unsafe {
                ShowWindow(self.hwnd, SW_HIDE);
            }
            ui.set_tray_hidden(true);
        }
        pub fn visible(&self) -> bool {
            unsafe { IsWindowVisible(self.hwnd) != 0 }
        }
        pub fn available(&self) -> bool {
            self.icon.rect().is_some()
        }
        pub fn update(&mut self, ui: &AppWindow) {
            let value = (
                ui.get_selected_id(),
                ui.get_playing(),
                ui.get_has_next(),
                ui.get_has_previous(),
                ui.get_selected_title().to_string(),
            );
            if self.last.as_ref() == Some(&value) {
                return;
            }
            self.toggle.set_text(if value.1 { "Pause" } else { "Play" });
            self.toggle.set_enabled(value.0 >= 0);
            self.next.set_enabled(value.2);
            self.previous.set_enabled(value.3);
            let text = if value.0 >= 0 {
                format!("MuseFlow · {}", value.4)
            } else {
                "MuseFlow PC".into()
            };
            let mut tooltip = String::new();
            let mut length = 0;
            for c in text.chars() {
                length += c.len_utf16();
                if length > 120 {
                    break;
                }
                tooltip.push(c);
            }
            let _ = self.icon.set_tooltip(Some(tooltip));
            self.last = Some(value);
        }
    }
    pub fn handlers(tx: SyncSender<Action>) {
        let menu = tx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if let Some(a) = action(event.id.as_ref()) {
                let _ = menu.try_send(a);
                let _ = slint::invoke_from_event_loop(|| {});
            }
        }));
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if event.id().as_ref() == "museflow.tray"
                && matches!(
                    event,
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } | TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    }
                )
            {
                let _ = tx.try_send(Action::Show);
                let _ = slint::invoke_from_event_loop(|| {});
            }
        }));
    }
}
impl Tray {
    #[cfg(windows)]
    pub fn new(ui: &AppWindow, engine: SyncSender<Command>) -> Self {
        use std::time::Duration;
        let (tx, rx) = mpsc::sync_channel(32);
        let callback = tx.clone();
        ui.on_tray_action(move |id| {
            if let Some(a) = action(id.as_str()) {
                let _ = callback.try_send(a);
            }
        });
        let weak = ui.as_weak();
        let timer = slint::Timer::default();
        let mut tray = None::<native::Native>;
        let mut attempts = 0;
        timer.start(slint::TimerMode::Repeated,Duration::from_millis(100),move||{
            let Some(ui)=weak.upgrade()else{return;};
            if ui.get_library_ready()&&tray.is_none()&&attempts<20{
                attempts+=1;
                let result=if std::env::args().any(|s|s=="--simulate-tray-failure"){Err("Simulated tray failure".into())}else{native::Native::new(&ui)};
                match result{Ok(t)=>{native::handlers(tx.clone());tray=Some(t);ui.set_tray_ready(true);ui.set_tray_status("Tray ready · left-click to show; right-click for controls and Exit.".into());},Err(e)=>{ui.set_tray_status(format!("Tray unavailable: {e}. Closing the window exits.").into());}}
            }
            while let Ok(a)=rx.try_recv(){
                match a{
                    Action::Show=>{if let Some(t)=&tray{t.show(&ui);}},
                    Action::Hide=>{if let Some(t)=&tray{t.hide(&ui);}},
                    Action::Exit=>{let _=slint::quit_event_loop();},
                    _=>{let command=match a{Action::Toggle=>Command::Toggle,Action::Next=>Command::Next,Action::Previous=>Command::Previous,_=>unreachable!()};if engine.try_send(command).is_err(){ui.set_message("The tray action could not be processed. Please try again.".into());}}
                }
            }
            if let Some(t)=&mut tray{
                let ready=t.available();
                if ui.get_tray_ready()!=ready{ui.set_tray_ready(ready);ui.set_tray_status(if ready{"Tray ready · left-click to show; right-click for controls and Exit."}else{"Tray is waiting for the Windows taskbar. Closing the window exits."}.into());}
                if !ready&&ui.get_tray_hidden(){t.show(&ui);}
                t.update(&ui);ui.set_native_window_visible(t.visible());
            }
        });
        Self { _timer: timer }
    }
    #[cfg(not(windows))]
    pub fn new(ui: &AppWindow, _engine: SyncSender<Command>) -> Self {
        ui.set_tray_status("Tray support is currently Windows-only. Closing exits.".into());
        Self {
            _timer: slint::Timer::default(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hiding_requires_opt_in_and_a_working_tray() {
        assert!(should_hide(true, true));
        for (pref, ready) in [(false, false), (false, true), (true, false)] {
            assert!(!should_hide(pref, ready));
        }
    }
    #[test]
    fn tray_commands_are_scoped_to_known_actions() {
        assert_eq!(action("museflow.exit"), Some(Action::Exit));
        assert_eq!(action("museflow.show"), Some(Action::Show));
        assert_eq!(action("museflow.previous"), Some(Action::Previous));
        assert!(action("open:https://example.com").is_none());
    }
}
