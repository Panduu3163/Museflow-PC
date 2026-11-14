//! Explicit opt-in live journeys with isolated --data-dir; real or null output is explicit.
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
    state: Rc<RefCell<Snapshot>>,
) -> Result<(slint::Timer, slint::Timer), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let mode = [
        "online",
        "online-saved",
        "online-restore",
        "online-cancel",
        "online-page",
        "online-switch",
        "online-no-runtime",
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
    if let Some((mode, out)) = mode {
        std::fs::create_dir_all(&out)?;
        let weak = ui.as_weak();
        let phase = Cell::new(0);
        let previous_count = Cell::new(0);
        let switched = RefCell::new(None::<Instant>);
        let name = RefCell::new(String::new());
        let id = RefCell::new(String::new());
        let start = Instant::now();
        let timeout = ui.as_weak();
        watchdog.start(
            slint::TimerMode::SingleShot,
            Duration::from_secs(90),
            move || {
                if let Some(ui) = timeout.upgrade() {
                    eprintln!(
                        "Online journey timeout: {} / {}; selected={} loaded={} playing={} position={} duration={}",
                        ui.get_message(),
                        ui.get_online_message(), ui.get_selected_id(), ui.get_audio_loaded(),
                        ui.get_playing(), ui.get_position(), ui.get_duration()
                    );
                }
                std::process::exit(1);
            },
        );
        timer.start(slint::TimerMode::Repeated,Duration::from_millis(100),move||{
            let Some(ui)=weak.upgrade()else{return;};if !ui.get_library_ready(){return;}
            let pass=|description:&str|{crate::demo::save_snapshot(&ui,out.join(format!("{mode}.png"))).unwrap();std::fs::write(out.join("PASS.txt"),format!("{description}\nElapsed {:.2}s. {} output; no acoustic claim.\n",start.elapsed().as_secs_f64(),if args.iter().any(|a|a=="--null-audio") {"Null"} else {"Default Windows (saved-recording fixture volume is zero)"})).unwrap();slint::quit_event_loop().unwrap();};
            if mode=="online-restore"{
                if ui.get_selected_id()<0{return;}assert!(!ui.get_playing()&&!ui.get_loading_audio()&&!ui.get_audio_loaded());
                assert!(state.borrow().tracks.iter().any(|t|t.id==ui.get_selected_id() as usize&&t.video_id.is_some()));pass("Online recording identity and queue restored paused without resolving a stream");return;
            }
            match phase.get(){
                0=>{
                    if mode=="online-saved" {
                        let wanted=args.iter().position(|s|s=="--online-recording-id").and_then(|n|args.get(n+1));
                        let track=state.borrow().tracks.iter().find(|t|t.video_id.as_ref().is_some_and(|id|wanted.is_none_or(|w|w==id))).cloned().expect("online-saved needs an isolated saved recording fixture");
                        *name.borrow_mut()=track.title.clone();
                        ui.invoke_change_volume(0.0);
                        ui.invoke_select_track(track.id as i32);phase.set(2);
                    } else { ui.invoke_navigate(1);ui.invoke_search_source(true);ui.set_query("Kevin MacLeod Carefree".into());ui.invoke_online_submit(false);phase.set(1); }
                }
                1=>{
                    if mode=="online-cancel"{ui.invoke_online_cancel();ui.invoke_navigate(2);phase.set(10);return;}
                    if ui.get_searching()||ui.get_tracks().row_count()==0{return;}
                    if mode=="online-page"{assert!(ui.get_online_more());previous_count.set(ui.get_tracks().row_count());*name.borrow_mut()=ui.get_tracks().row_data(0).unwrap().title.to_string();ui.invoke_online_submit(true);phase.set(11);return;}
                    let first=ui.get_tracks().row_data(0).unwrap();*name.borrow_mut()=first.title.to_string();ui.invoke_select_track(first.id);phase.set(2);
                }
                2=>{
                    if mode=="online-no-runtime"{if ui.get_message().contains("bundled resolver"){assert!(!ui.get_loading_audio()&&!ui.get_audio_loaded()&&!ui.get_playing());pass("Missing resolver runtime is a visible recoverable error");}return;}
                    if mode=="online-switch"{if !ui.get_loading_audio(){return;}ui.invoke_navigate(2);phase.set(12);return;}
                    if !ui.get_audio_loaded()||!ui.get_playing()||ui.get_position()<0.3{return;}
                    assert_eq!(ui.get_selected_title().as_str(),name.borrow().as_str());assert!(!ui.get_codec_label().is_empty());assert!(ui.get_duration()>30.0);
                    *id.borrow_mut()=state.borrow().tracks.iter().find(|t|t.id==ui.get_selected_id() as usize).unwrap().video_id.clone().unwrap();
                    ui.invoke_toggle_play();phase.set(3);
                }
                3=>{if ui.get_playing(){return;}ui.invoke_seek(if mode=="online-saved" {ui.get_duration()*0.6} else {10.0});ui.invoke_change_volume(0.0);*switched.borrow_mut()=Some(Instant::now());phase.set(4);}
                4=>{let target=if mode=="online-saved" {ui.get_duration()*0.6} else {10.0};if switched.borrow().unwrap().elapsed()<Duration::from_secs(2)||!ui.get_audio_loaded()||ui.get_loading_audio()||ui.get_playing()||(ui.get_position()-target).abs()>1.0{return;}
                    if mode=="online-saved" { ui.invoke_toggle_play(); *switched.borrow_mut()=Some(Instant::now()); phase.set(5); return; }
                    ui.invoke_toggle_like(ui.get_selected_id());ui.set_panel_open(true);
                    std::fs::write(out.join("recording.txt"),format!("{}\n{}\ncodec={}\n",id.borrow(),name.borrow(),ui.get_codec_label())).unwrap();pass("Live music search -> exact recording resolve -> decode/time advance -> pause/seek; stable recording ID saved, stream URL not persisted");
                }
                5=>{
                    if switched.borrow().unwrap().elapsed()<Duration::from_secs(3)||!ui.get_audio_loaded()||!ui.get_playing()||ui.get_position()<ui.get_duration()*0.6+1.0{return;}
                    ui.invoke_toggle_play(); *switched.borrow_mut()=Some(Instant::now()); phase.set(6);
                }
                6=>{
                    if switched.borrow().unwrap().elapsed()<Duration::from_secs(2)||!ui.get_audio_loaded()||ui.get_loading_audio()||ui.get_playing(){return;}
                    std::fs::write(out.join("recording.txt"),format!("{}\n{}\ncodec={}\n",id.borrow(),name.borrow(),ui.get_codec_label())).unwrap();
                    pass("Saved exact recording -> decode/time advance -> paused seek to 60% -> resume/time advance -> stable pause");
                }
                10=>{if start.elapsed()<Duration::from_secs(2){return;}assert_eq!(ui.get_page(),2);assert!(!ui.get_online_active());assert!(ui.get_tracks().row_count()==state.borrow().tracks.len());pass("Cancelling live search and switching to Library keeps stale online results out of the active view");}
                11=>{if ui.get_searching(){return;}assert!(ui.get_tracks().row_count()>previous_count.get());assert_eq!(ui.get_tracks().row_data(0).unwrap().title.as_str(),name.borrow().as_str());pass("Live continuation page appends results while retaining the first page");}
                12=>{if ui.get_searching()||ui.get_import_busy(){return;}let local=state.borrow().tracks.iter().find(|t|t.path.is_some()).map(|t|t.id as i32).expect("online-switch needs a local fixture");ui.invoke_select_track(local);phase.set(13);*switched.borrow_mut()=Some(Instant::now());}
                13=>{if switched.borrow().unwrap().elapsed()<Duration::from_secs(2){return;}assert!(!ui.get_loading_audio());assert!(ui.get_audio_loaded()&&ui.get_playing());assert!(ui.get_codec_label().starts_with("pcm"));assert!(state.borrow().tracks.iter().any(|t|t.id==ui.get_selected_id() as usize&&t.path.is_some()));pass("A local selection interrupts online resolution and remains the active decoded recording");}
                _=>{}
            }
        });
    }
    Ok((timer, watchdog))
}
