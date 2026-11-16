//! Opt-in native keyboard journey over synthetic metadata; no personal library/audio.
use crate::AppWindow;
use slint::{
    ComponentHandle, Model,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{cell::Cell, path::PathBuf, time::Duration};

fn key(ui: &AppWindow, text: slint::SharedString) {
    ui.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    ui.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}

pub fn install(ui: &AppWindow) -> Result<slint::Timer, Box<dyn std::error::Error>> {
    let timer = slint::Timer::default();
    let args: Vec<_> = std::env::args().collect();
    let Some(n) = args.iter().position(|s| s == "--list-keyboard-smoke-test") else {
        return Ok(timer);
    };
    ui.set_dark(!args.iter().any(|s| s == "--list-keyboard-light"));
    let output = PathBuf::from(
        args.get(n + 1)
            .ok_or("Missing list-keyboard output directory")?,
    );
    std::fs::create_dir_all(&output)?;
    let weak = ui.as_weak();
    let phase = Cell::new(0);
    let ticks = Cell::new(0);
    let tabs = Cell::new(0);
    timer.start(slint::TimerMode::Repeated, Duration::from_millis(150), move || {
        ticks.set(ticks.get() + 1);
        let Some(ui) = weak.upgrade() else { return; };
        if ticks.get() > 250 {
            eprintln!("List keyboard timeout: phase={} cursor={} selected={} rows={}", phase.get(), ui.get_list_cursor_id(), ui.get_selected_id(), ui.get_tracks().row_count());
            std::process::exit(1);
        }
        if ui.get_searching() { return; }
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            match phase.get() {
                0 => {
                    ui.window().set_size(slint::PhysicalSize::from_logical(slint::LogicalSize::new(820.0, 640.0),ui.window().scale_factor()));
                    let p = ui.get_library_button_origin();
                    let position = slint::LogicalPosition::new(p.x + 30.0, p.y + 18.0);
                    ui.window().dispatch_event(WindowEvent::PointerMoved { position });
                    ui.window().dispatch_event(WindowEvent::PointerPressed { position, button: PointerEventButton::Left });
                    ui.window().dispatch_event(WindowEvent::PointerReleased { position, button: PointerEventButton::Left });
                }
                1 | 10 => {
                    if ui.get_list_cursor_id() != 0 {
                        tabs.set(tabs.get()+1);assert!(tabs.get()<=32,"Tab must reach a track row");key(&ui,Key::Tab.into());return Ok(());
                    }
                    if phase.get()==1 {assert_eq!(ui.get_tracks().row_count(),10_000);key(&ui,Key::DownArrow.into());}
                    else {assert_eq!(ui.get_tracks().row_count(),1667);key(&ui,Key::DownArrow.into());}
                }
                2 => {assert_eq!(ui.get_list_cursor_id(),1);assert_eq!(ui.get_selected_id(),-1);key(&ui,Key::UpArrow.into());}
                3 => {assert_eq!(ui.get_list_cursor_id(),0);assert_eq!(ui.get_selected_id(),-1);key(&ui,Key::End.into());}
                4 => {assert_eq!(ui.get_list_cursor_id(),9999);assert_eq!(ui.get_selected_id(),-1);assert_complete_focus_outline(&ui);crate::demo::save_snapshot(&ui,output.join("last-row.png"))?;key(&ui,Key::Return.into());}
                5 => {assert_eq!(ui.get_selected_id(),9999);key(&ui,Key::Home.into());}
                6 => {assert_eq!(ui.get_list_cursor_id(),0);assert_eq!(ui.get_selected_id(),9999);key(&ui,Key::PageDown.into());}
                7 => {assert!(ui.get_list_cursor_id()>0&&ui.get_list_cursor_id()<20);assert_eq!(ui.get_selected_id(),9999);assert_complete_focus_outline(&ui);crate::demo::save_snapshot(&ui,output.join("page-down.png"))?;key(&ui,Key::PageUp.into());}
                8 => {assert_eq!(ui.get_list_cursor_id(),0);key(&ui,Key::DownArrow.into());ui.window().dispatch_event(WindowEvent::KeyPressRepeated{text:Key::DownArrow.into()});}
                9 => {assert_eq!(ui.get_list_cursor_id(),2);assert_eq!(ui.get_selected_id(),9999);ui.invoke_search_edited("VELVET".into());tabs.set(0);}
                11 => {assert_eq!(ui.get_list_cursor_id(),6);assert_eq!(ui.get_selected_id(),9999);key(&ui,Key::End.into());}
                12 => {assert_eq!(ui.get_list_cursor_id(),9996);assert_eq!(ui.get_selected_id(),9999);assert_complete_focus_outline(&ui);crate::demo::save_snapshot(&ui,output.join("filtered-last-row.png"))?;key(&ui," ".into());}
                13 => {assert_eq!(ui.get_selected_id(),9996);ui.invoke_search_edited("no such recording".into());}
                14 => {assert_eq!(ui.get_tracks().row_count(),0);assert_eq!(ui.get_list_cursor_id(),-1);assert_eq!(ui.get_selected_id(),9996);key(&ui,Key::DownArrow.into());ui.invoke_navigate(2);}
                15 => {assert_eq!(ui.get_tracks().row_count(),10_000);ui.window().set_size(slint::PhysicalSize::from_logical(slint::LogicalSize::new(800.0,600.0),ui.window().scale_factor()));ui.invoke_navigate(0);tabs.set(0);}
                16 => {assert_eq!(ui.get_tracks().row_count(),12);if ui.get_list_cursor_id()!=0{tabs.set(tabs.get()+1);assert!(tabs.get()<=32);key(&ui,Key::Tab.into());return Ok(());}key(&ui,Key::End.into());}
                17 => {assert_eq!(ui.get_list_cursor_id(),11);assert_eq!(ui.get_selected_id(),9996);assert_complete_focus_outline(&ui);crate::demo::save_snapshot(&ui,output.join("home-minimum-last-row.png"))?;}
                18 => {std::fs::write(output.join("PASS.txt"),"PASS: Tab entry, Up/Down cursor movement, Home/End across 10,000 rows, Page Up/Down, held-arrow navigation, Enter/Space activation, filtered stable IDs and empty results. Cursor movement does not change playback selection. Synthetic metadata only; screen-reader certification remains pending.\n")?;slint::quit_event_loop()?;}
                _ => unreachable!(),
            }
            phase.set(phase.get()+1);
            Ok(())
        })();
        if let Err(error)=result {eprintln!("List keyboard journey: {error}");std::process::exit(1);}
    });
    Ok(timer)
}

// Check the rendered top AND bottom of the wide focus ring. A cursor ID alone
// can pass while the final row is clipped behind the viewport/player boundary.
fn assert_complete_focus_outline(ui: &AppWindow) {
    let image = ui.window().take_snapshot().expect("native focus snapshot");
    let mut bands: Vec<(usize, usize)> = Vec::new();
    let accent = if ui.get_dark() {
        [197, 178, 255]
    } else {
        [100, 66, 188]
    };
    for (y, row) in image
        .as_bytes()
        .chunks_exact(image.width() as usize * 4)
        .enumerate()
    {
        let pixels = row
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| {
                p[0].abs_diff(accent[0]) <= 2
                    && p[1].abs_diff(accent[1]) <= 2
                    && p[2].abs_diff(accent[2]) <= 2
                    && p[3] == 255
            })
            .count();
        if pixels > image.width() as usize / 3 {
            if let Some(last) = bands.last_mut().filter(|last| last.1 + 1 == y) {
                last.1 = y;
            } else {
                bands.push((y, y));
            }
        }
    }
    let scale = ui.window().scale_factor();
    assert!(
        bands.windows(2).any(|pair| {
            let span = (pair[1].1 - pair[0].0 + 1) as f32;
            (span - 66.0 * scale).abs() <= 4.0 * scale
        }),
        "Focused row must show its complete top/bottom border: {bands:?}"
    );
}
