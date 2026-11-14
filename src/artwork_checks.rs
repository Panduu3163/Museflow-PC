//! Generated-fixture native cover checks, explicitly requested by CLI.
use crate::{AppWindow, library::Snapshot};
use slint::ComponentHandle;
use std::{
    cell::{Cell, RefCell},
    io::{Cursor, Write},
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant},
};
pub fn create(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use lofty::{
        config::WriteOptions,
        picture::{MimeType, Picture, PictureType},
        tag::{Accessor, Tag, TagExt, TagType},
    };
    for (folder, title, color) in [
        ("embedded", "Embedded cover", [220, 35, 75, 255]),
        ("folder", "Folder cover", [25, 90, 215, 255]),
        ("missing", "Missing cover", [0, 0, 0, 255]),
    ] {
        let dir = root.join(folder);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{title}.wav"));
        let bytes = 12u32 * 8000 * 2;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(b"RIFF")?;
        file.write_all(&(36 + bytes).to_le_bytes())?;
        file.write_all(b"WAVEfmt ")?;
        file.write_all(&16u32.to_le_bytes())?;
        file.write_all(&1u16.to_le_bytes())?;
        file.write_all(&1u16.to_le_bytes())?;
        file.write_all(&8000u32.to_le_bytes())?;
        file.write_all(&16000u32.to_le_bytes())?;
        file.write_all(&2u16.to_le_bytes())?;
        file.write_all(&16u16.to_le_bytes())?;
        file.write_all(b"data")?;
        file.write_all(&bytes.to_le_bytes())?;
        file.write_all(&vec![0; bytes as usize])?;
        drop(file);
        let mut tag = Tag::new(TagType::Id3v2);
        tag.set_title(title.into());
        tag.set_artist("MuseFlow generated fixture".into());
        let png = |color| -> Result<Vec<u8>, image::ImageError> {
            let mut out = Cursor::new(Vec::new());
            image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
                128,
                128,
                image::Rgba(color),
            ))
            .write_to(&mut out, image::ImageFormat::Png)?;
            Ok(out.into_inner())
        };
        if folder == "embedded" {
            tag.push_picture(
                Picture::unchecked(png([25, 90, 215, 255])?)
                    .mime_type(MimeType::Png)
                    .pic_type(PictureType::CoverBack)
                    .build(),
            );
            tag.push_picture(
                Picture::unchecked(png(color)?)
                    .mime_type(MimeType::Png)
                    .pic_type(PictureType::CoverFront)
                    .build(),
            );
            std::fs::write(dir.join("cover.png"), png([25, 90, 215, 255])?)?;
        } else if folder == "folder" {
            std::fs::write(dir.join("cover.png"), png(color)?)?;
        } else {
            std::fs::write(dir.join("cover.jpg"), b"damaged cover")?;
        }
        tag.save_to_path(path, WriteOptions::default())?;
    }
    Ok(())
}
pub fn install(
    ui: &AppWindow,
    state: Rc<RefCell<Snapshot>>,
) -> Result<(slint::Timer, slint::Timer), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let mode = [
        "artwork-local",
        "artwork-restore",
        "artwork-online",
        "artwork-online-restore",
    ]
    .into_iter()
    .find_map(|mode| {
        args.iter()
            .position(|a| a == &format!("--{mode}-smoke-test"))
            .and_then(|n| args.get(n + 1))
            .map(|p| (mode, PathBuf::from(p)))
    });
    let timer = slint::Timer::default();
    let watchdog = slint::Timer::default();
    if let Some((mode, out)) = mode {
        std::fs::create_dir_all(&out)?;
        let weak = ui.as_weak();
        let timeout = ui.as_weak();
        let start = Instant::now();
        let phase = Cell::new(0);
        let target = Cell::new(-1);
        watchdog.start(
            slint::TimerMode::SingleShot,
            Duration::from_secs(60),
            move || {
                if let Some(ui) = timeout.upgrade() {
                    eprintln!(
                        "Cover timeout: {} ready={} art_id={} selected={} origin={}",
                        ui.get_message(),
                        ui.get_artwork_ready(),
                        ui.get_artwork_track_id(),
                        ui.get_selected_id(),
                        ui.get_artwork_origin()
                    );
                }
                std::process::exit(1);
            },
        );
        timer.start(slint::TimerMode::Repeated,Duration::from_millis(100),move||{
            let Some(ui)=weak.upgrade()else{return;};if !ui.get_library_ready()||ui.get_import_busy()||ui.get_searching(){return;}
            let pass=|message:&str|{ui.set_panel_open(true);ui.set_lyrics_panel(true);crate::demo::save_snapshot(&ui,out.join(format!("{mode}.png"))).unwrap();std::fs::write(out.join("PASS.txt"),format!("{message}\nElapsed {:.2}s. Generated/muted output; no acoustic claim.\n",start.elapsed().as_secs_f64())).unwrap();slint::quit_event_loop().unwrap();};
            let ready=||ui.get_artwork_ready()&&ui.get_artwork_track_id()==ui.get_selected_id();
            let id=|name:&str|state.borrow().tracks.iter().find(|t|t.title==name).expect("Cover fixture missing").id as i32;
            let color=|red:bool|{let pixels=ui.get_artwork().to_rgba8().expect("Cover must have RGBA pixels");assert_eq!(pixels.width(),crate::artwork::SIDE);let p=pixels.as_bytes();assert!(if red{p[0]>p[2]}else{p[2]>p[0]});};
            if mode.ends_with("restore"){
                if !ready(){return;}assert!(!ui.get_playing()&&!ui.get_audio_loaded());assert_eq!(ui.get_artwork_origin(),"cached");
                if mode=="artwork-restore"{assert_eq!(ui.get_selected_title(),"Folder cover");color(false);}
                pass("Paused current-cover restored from the disk cache without audio initialization or online resolution");return;
            }
            if mode=="artwork-online"{
                if phase.get()==0{let track=state.borrow().tracks.iter().find(|t|t.video_id.is_some()).cloned().expect("Needs isolated saved online fixture");ui.invoke_change_volume(0.0);ui.invoke_select_track(track.id as i32);phase.set(1);return;}
                if phase.get()==1&&ready()&&ui.get_audio_loaded()&&ui.get_position()>0.3{assert_eq!(ui.get_artwork_origin(),"online");ui.invoke_toggle_play();phase.set(2);return;}
                if phase.get()==2&&!ui.get_playing(){pass("Online cover fetched/decoded for the exact selected recording alongside real playback");}return;
            }
            match phase.get(){
                0=>{ui.invoke_change_volume(0.0);ui.invoke_select_track(id("Embedded cover"));phase.set(1);}
                1=>{if !ready()||!ui.get_audio_loaded(){return;}assert_eq!(ui.get_artwork_origin(),"embedded");color(true);ui.invoke_select_track(id("Folder cover"));phase.set(2);}
                2=>{if !ready()||!ui.get_audio_loaded(){return;}assert_eq!(ui.get_artwork_origin(),"folder");color(false);ui.invoke_select_track(id("Missing cover"));phase.set(3);}
                3=>{if ui.get_artwork_track_id()!=ui.get_selected_id()||!ui.get_audio_loaded(){return;}assert!(!ui.get_artwork_ready());for n in 0..30{let choice=if n%2==0{id("Embedded cover")}else{id("Folder cover")};target.set(choice);ui.invoke_select_track(choice);}phase.set(4);}
                4=>{if !ready()||!ui.get_audio_loaded()||ui.get_selected_id()!=target.get(){return;}color(false);ui.invoke_toggle_play();phase.set(5);}
                5=>{if ui.get_playing(){return;}pass("Embedded front cover wins over back/folder art; folder PNG, damaged/missing fallback and thirty rapid selections retain correct final cover");}
                _=>{}
            }
        });
    }
    Ok((timer, watchdog))
}
