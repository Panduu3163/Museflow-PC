//! Opt-in native integration journeys. All fixture/data paths are supplied explicitly.
use crate::{AppWindow, library::Snapshot};
use slint::{ComponentHandle, Model};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

pub struct Checks {
    _poll: slint::Timer,
    _watchdog: slint::Timer,
}
fn argument(name: &str) -> Option<PathBuf> {
    let a = std::env::args().collect::<Vec<_>>();
    a.iter()
        .position(|s| s == name)
        .and_then(|n| a.get(n + 1))
        .map(PathBuf::from)
}
pub fn install(
    ui: &AppWindow,
    state: Rc<RefCell<Snapshot>>,
) -> Result<Checks, Box<dyn std::error::Error>> {
    let mode = [
        "local",
        "restore",
        "codec",
        "missing",
        "runtime-failure",
        "rapid",
        "startup-error",
    ]
    .into_iter()
    .find_map(|mode| argument(&format!("--{mode}-smoke-test")).map(|p| (mode, p)));
    let poll = slint::Timer::default();
    let watchdog = slint::Timer::default();
    if let Some((mode, out)) = mode {
        std::fs::create_dir_all(&out)?;
        let weak = ui.as_weak();
        let phase = Cell::new(0);
        let first = Cell::new(-1);
        let index = Cell::new(0usize);
        let started = Instant::now();
        let timeout_ui = ui.as_weak();
        watchdog.start(
            slint::TimerMode::SingleShot,
            Duration::from_secs(45),
            move || {
                if let Some(ui) = timeout_ui.upgrade() {
                    eprintln!(
                        "Local smoke timeout: {} selected={} loaded={} playing={} position={}",
                        ui.get_message(),
                        ui.get_selected_id(),
                        ui.get_audio_loaded(),
                        ui.get_playing(),
                        ui.get_position()
                    );
                }
                std::process::exit(1);
            },
        );
        poll.start(slint::TimerMode::Repeated,Duration::from_millis(150),move||{
            let Some(ui)=weak.upgrade()else{return;};if mode!="startup-error" && (!ui.get_library_ready()||ui.get_searching()||ui.get_import_busy()){return;}
            let result=(||->Result<(),Box<dyn std::error::Error>>{
                let pass=|description:&str|->Result<(),Box<dyn std::error::Error>>{
                    crate::demo::save_snapshot(&ui,out.join(format!("{mode}.png")))?;
                    std::fs::write(out.join("PASS.txt"),format!("{description}\nElapsed {:.2}s. Null output tests do not establish acoustic/device acceptance.\n",started.elapsed().as_secs_f64()))?;
                    slint::quit_event_loop()?;Ok(())
                };
                if mode=="startup-error"{
                    if ui.get_message().contains("already in use"){assert!(!ui.get_library_ready());return pass("A second instance cannot open and overwrite the active library");}
                    return Ok(());
                }
                if mode=="restore"{
                    if ui.get_selected_id()<0||ui.get_queue_tracks().row_count()<2{return Ok(());}
                    assert!(!ui.get_playing()&&!ui.get_audio_loaded()&&!ui.get_loading_audio());
                    assert!((ui.get_volume()-17.0).abs()<0.1);assert!(ui.get_shuffle_enabled());assert_eq!(ui.get_repeat_label(),"Repeat all");
                    assert!((ui.get_position()-2.0).abs()<0.4);
                    return pass("Paused queue, position, volume and repeat/shuffle restored without initializing audio");
                }
                if ui.get_tracks().row_count()<2 && (mode!="local" || phase.get()==0){return Ok(());}
                if mode=="codec"{
                    let count=ui.get_tracks().row_count();let n=index.get();
                    if phase.get()==0{ui.invoke_select_track(ui.get_tracks().row_data(n).unwrap().id);phase.set(1);return Ok(());}
                    let id=ui.get_tracks().row_data(n).unwrap().id;
                    if !ui.get_audio_loaded()||ui.get_selected_id()!=id||ui.get_position()<0.15{return Ok(());}
                    let s=state.borrow();let track=s.tracks.iter().find(|t|t.id==id as usize).unwrap();
                    let ext=track.path.as_ref().unwrap().extension().unwrap().to_str().unwrap();
                    let expected=match ext{"flac"=>"flac","mp3"=>"mp3","opus"=>"opus","m4a"=>"aac","wav"=>"pcm",_=>panic!("unexpected test format")};
                    let actual=ui.get_codec_label().to_string();assert!(actual.contains(expected),"{ext}: expected {expected}, got {actual}");
                    use std::io::Write;
                    let mut log=std::fs::OpenOptions::new().create(true).append(true).open(out.join("decoded-codecs.txt"))?;
                    writeln!(log,"{ext}: {actual}, duration={:.3}s",ui.get_duration())?;
                    if n+1==count{return pass("All imported codec fixtures decoded through the real player and advanced playback time");}
                    index.set(n+1);phase.set(0);return Ok(());
                }
                if mode=="runtime-failure"{
                    if phase.get()==0{ui.invoke_select_track(ui.get_tracks().row_data(0).unwrap().id);phase.set(1);}
                    else if ui.get_message().contains("Audio engine unavailable"){
                        assert!(!ui.get_audio_loaded()&&!ui.get_playing()&&!ui.get_loading_audio());return pass("Missing runtime produces a recoverable visible error, not an application crash");
                    }return Ok(());
                }
                if mode=="missing"{
                    match phase.get(){
                        0=>{first.set(ui.get_tracks().row_data(0).unwrap().id);ui.invoke_select_track(first.get());phase.set(1);}
                        1=>{if !ui.get_message().contains("File missing"){return Ok(());}assert!(!ui.get_playing()&&!ui.get_audio_loaded());ui.invoke_next_track();phase.set(2);}
                        _=>{if ui.get_audio_loaded()&&ui.get_playing()&&ui.get_selected_id()!=first.get(){return pass("Missing-file failure stays stopped; Next recovers to the correct valid track");}}
                    }return Ok(());
                }
                if mode=="rapid"{
                    if phase.get()==0{let rows=ui.get_tracks();for n in 0..30{let id=rows.row_data(n%2).unwrap().id;first.set(id);ui.invoke_select_track(id);}phase.set(1);}
                    else if ui.get_audio_loaded()&&ui.get_playing()&&ui.get_position()>0.2{
                        assert_eq!(ui.get_selected_id(),first.get());return pass("Thirty rapid selections settle on the final requested track with active decoding");
                    }return Ok(());
                }
                match phase.get(){
                    0=>{first.set(ui.get_tracks().row_data(0).unwrap().id);ui.invoke_select_track(first.get());phase.set(1);}
                    1=>{if !ui.get_audio_loaded()||ui.get_position()<0.25{return Ok(());}assert!(ui.get_playing());assert!(ui.get_duration()>4.0);ui.invoke_toggle_play();phase.set(2);}
                    2=>{if ui.get_playing(){return Ok(());}ui.invoke_seek(1.25);ui.invoke_change_volume(17.0);ui.invoke_toggle_like(first.get());ui.invoke_create_playlist("Integration playlist".into());phase.set(3);}
                    3=>{if (ui.get_position()-1.25).abs()>0.4||ui.get_playlist_names().row_count()<2{return Ok(());}ui.set_page(2);ui.invoke_collection_chosen(1);ui.invoke_add_selected();phase.set(4);}
                    4=>{if ui.get_tracks().row_count()!=1{return Ok(());}assert!(ui.get_tracks().row_data(0).unwrap().liked);ui.invoke_cycle_repeat();ui.invoke_toggle_shuffle();ui.invoke_next_track();phase.set(5);}
                    5=>{if !ui.get_audio_loaded()||ui.get_selected_id()==first.get(){return Ok(());}ui.invoke_previous_track();phase.set(6);}
                    6=>{if !ui.get_audio_loaded()||ui.get_selected_id()!=first.get(){return Ok(());}ui.invoke_cycle_repeat();ui.invoke_seek(ui.get_duration()-0.3);phase.set(7);}
                    7=>{if ui.get_position()<ui.get_duration()-0.7{return Ok(());}assert_eq!(ui.get_repeat_label(),"Repeat one");phase.set(8);}
                    8=>{if ui.get_loading_audio()||!ui.get_audio_loaded()||ui.get_position()>1.0{return Ok(());}assert_eq!(ui.get_selected_id(),first.get());assert!(ui.get_playing());ui.invoke_cycle_repeat();ui.invoke_cycle_repeat();ui.invoke_seek(ui.get_duration()-0.3);phase.set(9);}
                    9=>{if ui.get_position()<ui.get_duration()-0.7{return Ok(());}phase.set(10);}
                    10=>{if !ui.get_audio_loaded()||ui.get_selected_id()==first.get(){return Ok(());}ui.invoke_toggle_play();ui.invoke_seek(2.0);phase.set(11);}
                    11=>{if ui.get_playing()||(ui.get_position()-2.0).abs()>0.4{return Ok(());}assert_eq!(ui.get_repeat_label(),"Repeat all");assert!(ui.get_shuffle_enabled());ui.set_panel_open(true);return pass("Real decoding, progression, pause/seek/volume, Next/Previous, EOF repeat-one and automatic advance, likes/playlists and queue modes passed");}
                    _=>{}
                }Ok(())
            })();
            if let Err(error)=result{eprintln!("Local smoke: {error}");std::process::exit(1);}
        });
    }
    Ok(Checks {
        _poll: poll,
        _watchdog: watchdog,
    })
}
