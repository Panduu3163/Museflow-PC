//! Opt-in native desktop checks; generated sidecar content and isolated libraries only.
use crate::{AppWindow, library::Snapshot};
use slint::{ComponentHandle, Model};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};
pub fn install(
    ui: &AppWindow,
    _state: Rc<RefCell<Snapshot>>,
) -> Result<(slint::Timer, slint::Timer), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let mode = ["lyrics", "desktop", "media-journey", "stream-media"]
        .into_iter()
        .find_map(|s| {
            args.iter()
                .position(|v| v == &format!("--{s}-smoke-test"))
                .and_then(|n| args.get(n + 1))
                .map(|p| (s, PathBuf::from(p)))
        });
    let timer = slint::Timer::default();
    let watchdog = slint::Timer::default();
    if let Some((mode, out)) = mode {
        std::fs::create_dir_all(&out)?;
        let weak = ui.as_weak();
        let phase = Cell::new(0);
        let start = Instant::now();
        let timeout = ui.as_weak();
        watchdog.start(
            slint::TimerMode::SingleShot,
            Duration::from_secs(45),
            move || {
                if let Some(ui) = timeout.upgrade() {
                    eprintln!(
                        "Desktop journey timeout: {} / {} / {}",
                        ui.get_message(),
                        ui.get_lyric_status(),
                        ui.get_desktop_status()
                    );
                }
                std::process::exit(1);
            },
        );
        timer.start(slint::TimerMode::Repeated,Duration::from_millis(150),move||{
            let Some(ui)=weak.upgrade()else{return;};if !ui.get_library_ready()||ui.get_import_busy()||ui.get_searching(){return;}
            let pass=|text:&str|{crate::demo::save_snapshot(&ui,out.join(format!("{mode}.png"))).unwrap();std::fs::write(out.join("PASS.txt"),format!("{text}\nElapsed {:.2}s; null output, no acoustic claim.\n",start.elapsed().as_secs_f64())).unwrap();slint::quit_event_loop().unwrap();};
            if mode=="desktop"{if ui.get_desktop_ready(){pass("Windows native media controls attached successfully and published initial playback state");}return;}
            if mode=="media-journey"||mode=="stream-media"{
                std::fs::write(out.join("state.json"),serde_json::to_vec(&serde_json::json!({"title":ui.get_selected_title().to_string(),"playing":ui.get_playing(),"loaded":ui.get_audio_loaded(),"loading":ui.get_loading_audio(),"message":ui.get_message().to_string(),"position":ui.get_position(),"desktop_ready":ui.get_desktop_ready(),"status":ui.get_desktop_status().to_string()})).unwrap()).unwrap();
                if phase.get()==0&&ui.get_desktop_ready(){if mode=="stream-media"{ui.invoke_navigate(1);ui.invoke_search_source(true);ui.set_query("Kevin MacLeod Carefree".into());ui.invoke_online_submit(false);phase.set(10);}else if ui.get_tracks().row_count()>=2{ui.invoke_select_track(ui.get_tracks().row_data(0).unwrap().id);phase.set(1);}}
                if phase.get()==10&&!ui.get_searching()&&ui.get_tracks().row_count()>0{ui.invoke_select_track(ui.get_tracks().row_data(0).unwrap().id);phase.set(1);}
                if phase.get()==1&&ui.get_audio_loaded()&&ui.get_playing()&&ui.get_position()>0.2{std::fs::write(out.join("READY"),ui.get_selected_title().as_str()).unwrap();phase.set(2);}
                if phase.get()==2&&out.join("FINISH").exists(){pass("External Windows media session commands and metadata checked by the native harness");}return;
            }
            match phase.get(){
                0=>{if ui.get_tracks().row_count()<2{return;}ui.invoke_select_track(ui.get_tracks().row_data(0).unwrap().id);ui.set_panel_open(true);ui.set_lyrics_panel(true);phase.set(1);}
                1=>{if !ui.get_audio_loaded()||ui.get_lyric_busy()||ui.get_lyric_lines().row_count()<3{return;}assert_eq!(ui.get_lyric_source(),"Local lyrics sidecar");ui.invoke_toggle_play();ui.invoke_seek(2.5);phase.set(2);}
                2=>{if ui.get_lyric_current()!=1{return;}ui.invoke_seek(0.5);phase.set(3);}
                3=>{if ui.get_lyric_current()!=0{return;}ui.invoke_next_track();phase.set(4);}
                4=>{if ui.get_lyric_busy()||!ui.get_audio_loaded()||ui.get_selected_title().contains("01")||ui.get_lyric_lines().row_count()!=0{return;}assert!(ui.get_lyric_status().contains("Add title"));ui.invoke_previous_track();phase.set(5);}
                5=>{if ui.get_lyric_busy()||ui.get_lyric_lines().row_count()<3{return;}ui.set_dark(false);ui.window().set_size(slint::LogicalSize::new(820.0,650.0));phase.set(6);}
                6=>{if ui.window().size().width as f32/ui.window().scale_factor()>900.0{return;}pass("Generated UTF-8 LRC sidecar loads, timed highlight follows forward/backward seek, track changes clear stale lyrics and narrow/light panel renders");}
                _=>{}
            }
        });
    }
    Ok((timer, watchdog))
}
#[cfg(windows)]
pub fn media_probe(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    use windows::{
        Media::Control::GlobalSystemMediaTransportControlsSessionManager as Manager,
        Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
    };
    let command = args.get(2).ok_or("Missing media command")?;
    let expected = args.get(3).ok_or("Missing expected fixture title")?;
    let output = args.get(4).ok_or("Missing media probe output")?;
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)?;
    }
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let manager = Manager::RequestAsync()?.join()?;
        let sessions = manager.GetSessions()?;
        for i in 0..sessions.Size()? {
            let session = sessions.GetAt(i)?;
            let source = session.SourceAppUserModelId()?.to_string();
            if !source.to_lowercase().contains("museflow") {
                continue;
            }
            let properties = session.TryGetMediaPropertiesAsync()?.join()?;
            let title = properties.Title()?.to_string();
            if &title != expected {
                continue;
            }
            let accepted = match command.as_str() {
                "inspect" => true,
                "play" => session.TryPlayAsync()?.join()?,
                "pause" => session.TryPauseAsync()?.join()?,
                "next" => session.TrySkipNextAsync()?.join()?,
                "previous" => session.TrySkipPreviousAsync()?.join()?,
                "stop" => session.TryStopAsync()?.join()?,
                _ => return Err("Unsupported media probe command".into()),
            };
            let status = session.GetPlaybackInfo()?.PlaybackStatus()?.0;
            let position =
                session.GetTimelineProperties()?.Position()?.Duration as f64 / 10000000.0;
            std::fs::write(
                output,
                serde_json::to_vec_pretty(
                    &serde_json::json!({"source":source,"title":title,"artist":properties.Artist()?.to_string(),"accepted":accepted,"playback_status":status,"position":position}),
                )?,
            )?;
            return Ok(());
        }
        Err("The expected MuseFlow fixture media session is not available".into())
    })();
    unsafe {
        RoUninitialize();
    }
    result
}
#[cfg(not(windows))]
pub fn media_probe(_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    Err("Windows-only probe".into())
}
