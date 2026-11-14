use std::collections::BTreeSet;

/// Synthetic metadata for shell testing. No audio, remote media or personal library.
#[derive(Clone, Debug)]
pub struct Track {
    pub id: usize,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: String,
    pub cover: usize,
    pub path: Option<std::path::PathBuf>,
    pub video_id: Option<String>,
    pub seconds: f64,
    search_key: String,
}

pub fn sample_catalog(count: usize) -> Vec<Track> {
    let names = [
        "Velvet Hours",
        "Tide Pools",
        "After the Rain",
        "Blue Orbit",
        "Golden Haze",
        "Paper Moons",
    ];
    let artists = [
        "The Quiet Lines",
        "North of June",
        "Amber Fields",
        "Satellite Bloom",
        "Sundial",
        "Evening Atlas",
    ];
    (0..count)
        .map(|id| {
            let cover = id % names.len();
            let title = if id < names.len() {
                names[cover].to_owned()
            } else {
                format!("{} · {:05}", names[cover], id + 1)
            };
            let artist = artists[cover].to_owned();
            let album = [
                "Midnight Studies",
                "Coastal Motion",
                "Soft Focus",
                "Distant Signals",
                "Sunday Light",
                "Small Worlds",
            ][cover]
                .to_owned();
            let search_key = format!("{title} {artist} {album}").to_lowercase();
            Track {
                id,
                title,
                artist,
                album,
                duration: format!("{}:{:02}", 3 + id % 3, id * 7 % 60),
                cover,
                path: None,
                video_id: None,
                seconds: 0.0,
                search_key,
            }
        })
        .collect()
}

impl Track {
    pub fn online(id: usize, song: &crate::online::Song) -> Self {
        let mut t = Self::local(
            id,
            std::path::PathBuf::new(),
            song.title.clone(),
            song.artist.clone(),
            song.album.clone(),
            song.seconds,
        );
        t.path = None;
        t.video_id = Some(song.video_id.clone());
        t
    }
    pub fn local(
        id: usize,
        path: std::path::PathBuf,
        title: String,
        artist: String,
        album: String,
        seconds: f64,
    ) -> Self {
        let search_key = format!("{title} {artist} {album}").to_lowercase();
        Self {
            id,
            path: Some(path),
            video_id: None,
            title,
            artist,
            album,
            duration: time_label(seconds),
            seconds,
            cover: usize::MAX,
            search_key,
        }
    }
}
pub fn time_label(seconds: f64) -> String {
    let n = if seconds.is_finite() {
        seconds.max(0.0) as u64
    } else {
        0
    };
    if n >= 3600 {
        format!("{}:{:02}:{:02}", n / 3600, n / 60 % 60, n % 60)
    } else {
        format!("{}:{:02}", n / 60, n % 60)
    }
}

pub fn filter_tracks(
    tracks: &[Track],
    page: i32,
    query: &str,
    likes: &BTreeSet<usize>,
    cancelled: impl Fn() -> bool,
) -> Option<Vec<usize>> {
    let query = query.trim().to_lowercase();
    let terms: Vec<_> = query.split_whitespace().collect();
    let mut indices = Vec::new();
    if page == 4 {
        return Some(indices);
    }
    for (index, track) in tracks.iter().enumerate() {
        if index % 128 == 0 && cancelled() {
            return None;
        }
        if page == 3 && !likes.contains(&track.id) {
            continue;
        }
        if terms.iter().all(|term| track.search_key.contains(term)) {
            indices.push(index);
            if page == 0 && terms.is_empty() && indices.len() == 12 {
                break;
            }
        }
    }
    Some(indices)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_library_retains_identity_when_filtered() {
        let tracks = sample_catalog(10_000);
        let indices =
            filter_tracks(&tracks, 1, "VELVET   quiet", &BTreeSet::new(), || false).unwrap();
        assert_eq!(indices.len(), 1667);
        assert_eq!(tracks[indices[1]].id, 6);
        assert_eq!(tracks[indices[1666]].id, 9996);
    }
    #[test]
    fn liked_view_obeys_ids_and_search_without_reordering() {
        let tracks = sample_catalog(10_000);
        let likes = BTreeSet::from([2, 6, 9999]);
        assert_eq!(
            filter_tracks(&tracks, 3, "", &likes, || false).unwrap(),
            vec![2, 6, 9999]
        );
        assert_eq!(
            filter_tracks(&tracks, 3, "velvet", &likes, || false).unwrap(),
            vec![6]
        );
    }
    #[test]
    fn cancelled_search_does_not_produce_partial_results() {
        let tracks = sample_catalog(10_000);
        let polls = std::cell::Cell::new(0);
        assert!(
            filter_tracks(&tracks, 2, "", &BTreeSet::new(), || {
                polls.set(polls.get() + 1);
                polls.get() == 2
            })
            .is_none()
        );
    }
    #[test]
    fn home_is_bounded_but_library_is_complete_and_empty_search_recovers() {
        let tracks = sample_catalog(10_000);
        let likes = BTreeSet::new();
        assert_eq!(
            filter_tracks(&tracks, 0, "", &likes, || false)
                .unwrap()
                .len(),
            12
        );
        assert_eq!(
            filter_tracks(&tracks, 2, "", &likes, || false)
                .unwrap()
                .len(),
            10_000
        );
        assert!(
            filter_tracks(&tracks, 1, "not a real title", &likes, || false)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            filter_tracks(&tracks, 1, "   ", &likes, || false)
                .unwrap()
                .len(),
            10_000
        );
    }
}
