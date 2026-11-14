//! Opt-in native routing/stress journeys. Caller supplies an isolated library and fixtures.
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
        "audio-output",
        "audio-restore",
        "audio-missing",
        "audio-stress",
    ]
    .into_iter()
    .find_map(|s| {
        args.iter()
            .position(|a| a == &format!("--{s}-smoke-test"))
            .and_then(|n| args.get(n + 1))
            .map(|p| (s, PathBuf::from(p)))
    });
    let timer = slint::Timer::default();
    let watchdog = slint::Timer::default();
    if let Some((mode, out)) = mode {
        std::fs::create_dir_all(&out)?;
        let weak = ui.as_weak();
        let phase = Cell::new(0);
        let count = Cell::new(0usize);
        let target = Cell::new(-1);
        let endpoint = RefCell::new(String::new());
        let seek_at = Cell::new(0.0);
        let changed = RefCell::new(Instant::now());
        let started = Instant::now();
        let timeout = ui.as_weak();
        watchdog.start(
            slint::TimerMode::SingleShot,
            Duration::from_secs(100),
            move || {
                if let Some(ui) = timeout.upgrade() {
                    eprintln!(
                        "Audio check timed out: {} / {}; loaded={} playing={} position={}",
                        ui.get_message(),
                        ui.get_audio_output_status(),
                        ui.get_audio_loaded(),
                        ui.get_playing(),
                        ui.get_position()
                    );
                }
                std::process::exit(1);
            },
        );
        timer.start(slint::TimerMode::Repeated,Duration::from_millis(100),move||{
            let Some(ui)=weak.upgrade()else{return;};if !ui.get_library_ready()||ui.get_import_busy(){return;}
            let pass=|description:&str|{
                crate::demo::save_snapshot(&ui,out.join(format!("{mode}.png"))).unwrap();
                std::fs::write(out.join("PASS.txt"),format!("{description}\nElapsed {:.2}s. Default audio output at zero app volume; no acoustic claim.\n",started.elapsed().as_secs_f64())).unwrap();
                slint::quit_event_loop().unwrap();
            };
            let selected=||ui.get_audio_output_ids().row_data(ui.get_audio_output_index() as usize).map(|s|s.to_string()).unwrap_or_default();
            match phase.get(){
                0=>{ui.invoke_navigate(4);ui.invoke_change_volume(0.0);phase.set(1);}
                1=>{
                    if !ui.get_audio_outputs_ready(){return;}
                    if mode=="audio-restore" {
                        assert!(selected().starts_with("wasapi/"));assert!(!ui.get_playing()&&!ui.get_audio_loaded());pass("Saved concrete output restored while queue remains paused/unloaded");return;
                    }
                    if mode=="audio-missing" {
                        assert!(ui.get_audio_output_status().contains("unavailable"));assert_eq!(selected(),"wasapi/museflow-unavailable-fixture");
                    } else if mode=="audio-output" {
                        let choice=ui.get_audio_output_ids().row_data(1).expect("Routing test needs a WASAPI endpoint").to_string();
                        assert!(choice.starts_with("wasapi/"));*endpoint.borrow_mut()=choice.clone();ui.invoke_choose_audio_output(choice.into());
                    }
                    phase.set(2);
                }
                2=>{
                    if mode=="audio-output" && selected()!=*endpoint.borrow(){return;}
                    let id=state.borrow().tracks.iter().find(|t|t.path.is_some()).expect("Audio checks need local fixtures").id as i32;
                    target.set(id);ui.invoke_navigate(2);phase.set(21);
                }
                21=>{
                    if ui.get_searching()||ui.get_tracks().row_count()<2{return;}
                    ui.invoke_select_track(target.get());phase.set(3);
                }
                3=>{
                    if !ui.get_audio_loaded()||!ui.get_playing()||ui.get_position()<0.3{return;}
                    if mode=="audio-missing" {assert!(ui.get_audio_output_status().contains("unavailable"));pass("Unavailable saved output falls back to default and decodes without erasing its preference");return;}
                    if mode=="audio-stress" {
                        count.set(count.get()+1);
                        if count.get()>=120 {ui.invoke_toggle_play();phase.set(20);return;}
                        let tracks=state.borrow().tracks.clone();let index=count.get()%tracks.len();target.set(tracks[index].id as i32);
                        ui.invoke_select_track(target.get());phase.set(10);return;
                    }
                    ui.invoke_choose_audio_output("wasapi/stale-unlisted-fixture".into());phase.set(4);
                }
                4=>{
                    if !ui.get_message().contains("no longer available"){return;}
                    assert!(ui.get_audio_loaded()&&ui.get_playing());assert_eq!(selected(),*endpoint.borrow());
                    ui.invoke_toggle_play();*changed.borrow_mut()=Instant::now();phase.set(5);
                }
                5=>{
                    if ui.get_playing()||changed.borrow().elapsed()<Duration::from_secs(1){return;}seek_at.set(ui.get_position());ui.invoke_choose_audio_output("auto".into());*changed.borrow_mut()=Instant::now();phase.set(6);
                }
                6=>{
                    if selected()!="auto"||changed.borrow().elapsed()<Duration::from_secs(1){return;}
                    assert!(!ui.get_playing()&&ui.get_audio_loaded());assert!((ui.get_position()-seek_at.get()).abs()<0.4,"Paused position moved: {} -> {}",seek_at.get(),ui.get_position());
                    ui.invoke_choose_audio_output(endpoint.borrow().clone().into());phase.set(7);
                }
                7=>{
                    if selected()!=*endpoint.borrow(){return;}ui.invoke_toggle_play();phase.set(8);
                }
                8=>{
                    if !ui.get_playing()||ui.get_position()<seek_at.get()+0.5{return;}
                    ui.invoke_next_track();phase.set(9);
                }
                9=>{
                    if !ui.get_audio_loaded()||ui.get_selected_id()==target.get()||ui.get_position()<0.3{return;}
                    assert_eq!(selected(),*endpoint.borrow());ui.invoke_toggle_play();phase.set(20);
                }
                10=>{
                    if ui.get_selected_id()==target.get()&&ui.get_audio_loaded()&&ui.get_playing()&&ui.get_position()>0.3 {phase.set(3);}
                }
                20=>{
                    if ui.get_playing(){return;}
                    pass(if mode=="audio-stress" {"120 decoded track selections settled on requested identities; normal shutdown"}else{"Concrete/default output switching, stale selection recovery, pause preservation, resume and Next passed"});
                }
                _=>{}
            }
        });
    }
    Ok((timer, watchdog))
}
