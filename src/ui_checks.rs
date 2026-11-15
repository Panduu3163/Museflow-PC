//! Opt-in compact-layout journey; use an isolated library, never personal data.
use crate::AppWindow;
use slint::ComponentHandle;
use std::{cell::Cell, path::PathBuf, time::Duration};

pub fn install(ui: &AppWindow) -> Result<slint::Timer, Box<dyn std::error::Error>> {
    let timer = slint::Timer::default();
    let args: Vec<_> = std::env::args().collect();
    let Some(index) = args.iter().position(|s| s == "--ui-smoke-test") else {
        return Ok(timer);
    };
    let output = PathBuf::from(args.get(index + 1).ok_or("Missing UI output directory")?);
    std::fs::create_dir_all(&output)?;
    let weak = ui.as_weak();
    let step = Cell::new(0usize);
    let ticks = Cell::new(0usize);
    timer.start(slint::TimerMode::Repeated, Duration::from_millis(250), move || {
        ticks.set(ticks.get() + 1);
        if ticks.get() > 160 {
            eprintln!("UI journey timed out");
            std::process::exit(1);
        }
        let Some(ui) = weak.upgrade() else { return; };
        if !ui.get_library_ready() || ui.get_searching() || ui.get_import_busy() { return; }
        let cases = [
            (800.0, 600.0, 4, true, false, false, "settings-dark-minimum"),
            (800.0, 600.0, 4, false, false, false, "settings-light-minimum"),
            (800.0, 600.0, 0, true, false, false, "home-minimum"),
            (800.0, 600.0, 2, true, false, false, "library-minimum"),
            (800.0, 600.0, 2, true, true, false, "queue-minimum"),
            (800.0, 600.0, 2, true, true, true, "lyrics-minimum"),
            (1240.0, 820.0, 2, true, true, false, "queue-wide"),
            (1240.0, 820.0, 2, false, false, false, "library-light-wide"),
        ];
        let n = step.get();
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            if n / 2 == cases.len() {
                check_keyboard(&ui);
                std::fs::write(output.join("PASS.txt"), "PASS: Ctrl+1–5 navigation, modified shortcut rejection, Escape panel dismissal, focused-button Return, ignored held-key repeats; eight isolated native layout captures. Images require visual inspection; this is not screen-reader or OS-DPI certification.\n")?;
                slint::quit_event_loop()?;
            } else {
                let (width, height, page, dark, panel, lyrics, name) = cases[n / 2];
                if n.is_multiple_of(2) {
                    ui.window().set_size(slint::PhysicalSize::from_logical(
                        slint::LogicalSize::new(width, height), ui.window().scale_factor(),
                    ));
                    ui.set_dark(dark);
                    ui.invoke_navigate(page);
                    ui.set_panel_open(panel);
                    ui.set_lyrics_panel(lyrics);
                } else {
                    let expected = slint::PhysicalSize::from_logical(
                        slint::LogicalSize::new(width, height), ui.window().scale_factor(),
                    );
                    assert_eq!(ui.window().size(), expected, "Scaled window must retain its requested logical content size");
                    crate::demo::save_snapshot(&ui, output.join(format!("{name}.png")))?;
                }
            }
            Ok(())
        })();
        if let Err(error) = result { eprintln!("UI journey: {error}"); std::process::exit(1); }
        step.set(n + 1);
    });
    Ok(timer)
}

fn check_keyboard(ui: &AppWindow) {
    use slint::platform::{Key, PointerEventButton, WindowEvent};
    let key = |text: slint::SharedString| {
        ui.window()
            .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
        ui.window()
            .dispatch_event(WindowEvent::KeyReleased { text });
    };
    let focus_library = || {
        let origin = ui.get_library_button_origin();
        let position = slint::LogicalPosition::new(origin.x + 30.0, origin.y + 18.0);
        ui.window()
            .dispatch_event(WindowEvent::PointerMoved { position });
        ui.window().dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
        ui.window().dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
        assert_eq!(ui.get_page(), 2);
    };
    focus_library();
    for page in 0..5 {
        ui.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Control.into(),
        });
        key((page + 1).to_string().into());
        ui.window().dispatch_event(WindowEvent::KeyReleased {
            text: Key::Control.into(),
        });
        assert_eq!(
            ui.get_page(),
            page,
            "Ctrl+number must work from focused controls"
        );
    }
    ui.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    ui.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    key("1".into());
    ui.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    ui.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    assert_eq!(
        ui.get_page(),
        4,
        "Ctrl+Shift+number is not a section shortcut"
    );
    ui.set_panel_open(true);
    key(Key::Escape.into());
    assert!(!ui.get_panel_open(), "Escape must close the side panel");

    focus_library();
    ui.set_query("held-key sentinel".into());
    ui.window().dispatch_event(WindowEvent::KeyPressRepeated {
        text: Key::Return.into(),
    });
    assert_eq!(
        ui.get_query(),
        "held-key sentinel",
        "Held Return must not reactivate the focused button"
    );
    key(Key::Return.into());
    assert!(
        ui.get_query().is_empty(),
        "A fresh Return must activate the focused button"
    );
}
