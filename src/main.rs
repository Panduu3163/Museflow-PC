#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
mod artwork;
mod artwork_checks;
mod artwork_ui;
mod audio_checks;
mod audio_outputs;
mod catalog;
mod demo;
mod desktop;
mod desktop_checks;
mod engine;
mod library;
mod local;
mod local_checks;
mod lyrics;
mod lyrics_ui;
mod online;
mod online_checks;
mod online_ui;
mod playback;
mod queue;
mod resolver;
mod tray;
mod tray_checks;
slint::include_modules!();
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args
        .get(1)
        .is_some_and(|a| a == "--artwork-create-fixtures")
    {
        return artwork_checks::create(std::path::Path::new(
            args.get(2).ok_or("Missing fixture directory")?,
        ));
    }
    if args.get(1).is_some_and(|a| a == "--audio-runtime-probe") {
        let output = args.get(2).ok_or("Missing audio probe output path")?;
        let mpv = playback::Mpv::open(&local::runtime_path()?, true)?;
        let values = [
            "cache-secs",
            "demuxer-max-bytes",
            "demuxer-max-back-bytes",
            "curl-max-request-size",
        ]
        .into_iter()
        .map(|key| (key, mpv.string(&format!("options/{key}"))))
        .collect::<std::collections::BTreeMap<_, _>>();
        std::fs::write(output, serde_json::to_vec_pretty(&values)?)?;
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "--media-probe") {
        return desktop_checks::media_probe(&args);
    }
    if args.get(1).is_some_and(|a| a == "--lyrics-provider-probe") {
        let path = args.get(2).ok_or("Missing probe output path")?;
        let track = catalog::Track::online(
            1,
            &online::Song {
                video_id: "abcdefghijk".into(),
                title: "Never Gonna Give You Up".into(),
                artist: "Rick Astley".into(),
                album: String::new(),
                seconds: 213.0,
            },
        );
        let doc = lyrics::Client::new()?.lookup(
            &track,
            std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
            0,
        )?;
        std::fs::write(
            path,
            serde_json::to_vec_pretty(
                &serde_json::json!({"source":doc.source,"lines":doc.lines.len(),"timed":doc.lines.first().is_some_and(|l|l.at.is_some()),"status":doc.status}),
            )?,
        )?;
        return Ok(());
    }
    if args
        .iter()
        .any(|a| matches!(a.as_str(), "--demo" | "--smoke-test" | "--benchmark-filter"))
    {
        demo::run()
    } else {
        local::run()
    }
}
