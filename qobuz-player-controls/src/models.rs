use qobuz_player_client::qobuz_models::{
    album::Album as QobuzAlbum,
    album_suggestion::AlbumSuggestion,
    artist::Artist as QobuzArtist,
    artist_page::{self, ArtistPage as QobuzArtistPage},
    playlist::Playlist as QobuzPlaylist,
    release::{Release, Track as QobuzReleaseTrack},
    search_results::SearchAllResults,
    track::Track as QobuzTrack,
};
use std::{collections::BTreeMap, fmt::Debug, str::FromStr};

pub fn parse_search_results(search_results: SearchAllResults, user_id: i64) -> SearchResults {
    SearchResults {
        query: search_results.query,
        albums: search_results
            .albums
            .items
            .into_iter()
            .map(|a| a.into())
            .collect(),
        artists: search_results
            .artists
            .items
            .into_iter()
            .map(|a| a.into())
            .collect(),
        playlists: search_results
            .playlists
            .items
            .into_iter()
            .map(|p| parse_playlist(p, user_id))
            .collect(),
        tracks: search_results
            .tracks
            .items
            .into_iter()
            .map(|t| t.into())
            .collect(),
    }
}

impl From<QobuzReleaseTrack> for Track {
    fn from(value: QobuzReleaseTrack) -> Self {
        Self {
            id: value.id,
            number: value.physical_support.track_number,
            title: value.title,
            album: None,
            artist: Some(Artist {
                id: value.artist.id,
                name: value.artist.name.display,
                ..Default::default()
            }),
            duration_seconds: value.duration as u32,
            explicit: value.parental_warning,
            hires_available: value.rights.streamable,
            cover_art: None,
            cover_art_small: None,
        }
    }
}

impl From<Release> for AlbumPage {
    fn from(s: Release) -> Self {
        let year = chrono::NaiveDate::from_str(&s.dates.original)
            .expect("failed to parse date")
            .format("%Y");

        let tracks: BTreeMap<u32, Track> = s.tracks.map_or(Default::default(), |tracks| {
            tracks
                .items
                .into_iter()
                .enumerate()
                .map(|(i, t)| (i as u32, t.into()))
                .collect()
        });

        Self {
            id: s.id,
            title: s.title,
            artist: Artist {
                id: s.artist.id,
                name: s.artist.name.display,
                ..Default::default()
            },
            release_year: year
                .to_string()
                .parse::<u32>()
                .expect("error converting year"),
            hires_available: s.rights.hires_streamable,
            explicit: s.parental_warning,
            total_tracks: s.tracks_count as u32,
            tracks,
            available: s.rights.streamable,
            cover_art: s.image.large,
            cover_art_small: s.image.small,
            duration_seconds: s.duration.map_or(0, |duration| duration as u32),
        }
    }
}

impl From<AlbumPage> for Album {
    fn from(value: AlbumPage) -> Self {
        Self {
            id: value.id,
            title: value.title,
            artist: value.artist,
            image: value.cover_art,
        }
    }
}

impl From<AlbumSuggestion> for AlbumPage {
    fn from(s: AlbumSuggestion) -> Self {
        let year = chrono::NaiveDate::from_str(&s.dates.original)
            .expect("failed to parse date")
            .format("%Y");

        let tracks = BTreeMap::new();

        let artist = s.artists.and_then(|vec| vec.into_iter().next());
        let (artist_id, artist_name) = artist.map_or((0, "Unknown".into()), |artist| {
            (artist.id as u32, artist.name)
        });

        Self {
            id: s.id,
            title: s.title,
            artist: Artist {
                id: artist_id,
                name: artist_name,
                ..Default::default()
            },
            release_year: year
                .to_string()
                .parse::<u32>()
                .expect("error converting year"),
            hires_available: s.rights.hires_streamable,
            explicit: s.parental_warning,
            total_tracks: s.track_count as u32,
            tracks,
            available: s.rights.streamable,
            cover_art: s.image.large,
            cover_art_small: s.image.small,
            duration_seconds: s.duration.map_or(0, |duration| duration as u32),
        }
    }
}

impl From<QobuzAlbum> for AlbumPage {
    fn from(value: QobuzAlbum) -> Self {
        let year = chrono::NaiveDate::from_str(&value.release_date_original)
            .expect("failed to parse date")
            .format("%Y");

        let tracks: BTreeMap<u32, Track> = value.tracks.map_or(Default::default(), |tracks| {
            tracks
                .items
                .into_iter()
                .enumerate()
                .map(|(i, t)| (i as u32, t.into()))
                .collect()
        });

        Self {
            id: value.id,
            title: value.title,
            artist: value.artist.into(),
            total_tracks: value.tracks_count as u32,
            release_year: year
                .to_string()
                .parse::<u32>()
                .expect("error converting year"),
            hires_available: value.hires_streamable,
            explicit: value.parental_warning,
            available: value.streamable,
            tracks,
            cover_art: value.image.large,
            cover_art_small: value.image.small,
            duration_seconds: value.duration.map_or(0, |duration| duration as u32),
        }
    }
}

fn image_to_string(value: artist_page::Image) -> String {
    format!(
        "https://static.qobuz.com/images/artists/covers/large/{}.{}",
        value.hash, value.format
    )
}

impl From<QobuzArtistPage> for ArtistPage {
    fn from(value: QobuzArtistPage) -> Self {
        let artist_image_url = value.images.portrait.map(image_to_string);

        Self {
            id: value.id,
            name: value.name.display.clone(),
            image: artist_image_url.clone(),
            top_tracks: value
                .top_tracks
                .into_iter()
                .map(|t| {
                    let album_image_url = t.album.image.large;
                    let album_image_url_small = t.album.image.small;
                    let artist = Artist {
                        id: value.id,
                        name: value.name.display.clone(),
                        image: artist_image_url.clone(),
                    };
                    Track {
                        id: t.id,
                        number: t.physical_support.track_number,
                        title: t.title,
                        album: Some(Album {
                            id: t.album.id,
                            title: t.album.title,
                            artist: artist.clone(),
                            image: album_image_url.clone(),
                        }),
                        artist: Some(artist),
                        duration_seconds: t.duration,
                        explicit: t.parental_warning,
                        hires_available: t.rights.hires_streamable,
                        cover_art: Some(album_image_url),
                        cover_art_small: Some(album_image_url_small),
                    }
                })
                .collect(),
        }
    }
}

impl From<QobuzArtist> for Artist {
    fn from(value: QobuzArtist) -> Self {
        Self {
            id: value.id,
            name: value.name,
            image: value.image.map(|i| i.large),
        }
    }
}

pub fn parse_playlist(playlist: QobuzPlaylist, user_id: i64) -> Playlist {
    let tracks: BTreeMap<u32, Track> = playlist.tracks.map_or(Default::default(), |tracks| {
        tracks
            .items
            .into_iter()
            .enumerate()
            .map(|(i, t)| (i as u32, t.into()))
            .collect()
    });

    let cover_art = if let Some(image) = playlist.image_rectangle.first() {
        Some(image.clone())
    } else if let Some(images) = playlist.images300 {
        images.first().cloned()
    } else {
        None
    };

    Playlist {
        id: playlist.id as u32,
        is_owned: user_id == playlist.owner.id,
        title: playlist.name,
        duration_seconds: playlist.duration as u32,
        tracks_count: playlist.tracks_count as u32,
        cover_art,
        tracks,
    }
}

impl From<QobuzTrack> for Track {
    fn from(value: QobuzTrack) -> Self {
        let artist = if let Some(p) = &value.performer {
            Some(Artist {
                id: p.id as u32,
                name: p.name.clone(),
                image: None,
            })
        } else {
            value.album.as_ref().map(|a| a.clone().artist.into())
        };

        let cover_art = value.album.as_ref().map(|a| a.image.large.clone());
        let cover_art_small = value.album.as_ref().map(|a| a.image.small.clone());

        let album = value.album.map(|a| Album {
            id: a.id,
            title: a.title,
            artist: a.artist.into(),
            image: a.image.small,
        });

        Self {
            id: value.id,
            number: value.track_number as u32,
            title: value.title,
            album,
            artist,
            duration_seconds: value.duration as u32,
            explicit: value.parental_warning,
            hires_available: value.hires_streamable,
            cover_art,
            cover_art_small,
        }
    }
}

impl From<&QobuzTrack> for Track {
    fn from(value: &QobuzTrack) -> Self {
        value.clone().into()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum TrackStatus {
    Played,
    Playing,
    #[default]
    Unplayed,
    Unplayable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: u32,
    pub number: u32,
    pub title: String,
    pub album: Option<Album>,
    pub artist: Option<Artist>,
    pub duration_seconds: u32,
    pub explicit: bool,
    pub hires_available: bool,
    pub cover_art: Option<String>,
    pub cover_art_small: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlbumPage {
    pub id: String,
    pub title: String,
    pub artist: Artist,
    pub release_year: u32,
    pub hires_available: bool,
    pub explicit: bool,
    pub total_tracks: u32,
    pub tracks: BTreeMap<u32, Track>,
    pub available: bool,
    pub cover_art: String,
    pub cover_art_small: String,
    pub duration_seconds: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: Artist,
    pub image: String,
}

#[derive(Default, Debug, Clone)]
pub struct SearchResults {
    pub query: String,
    pub albums: Vec<AlbumPage>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
    pub tracks: Vec<Track>,
}

#[derive(Default, Debug, Clone)]
pub struct Favorites {
    pub albums: Vec<AlbumPage>,
    pub artists: Vec<Artist>,
    pub playlists: Vec<Playlist>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Artist {
    pub id: u32,
    pub name: String,
    pub image: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ArtistPage {
    pub id: u32,
    pub name: String,
    pub image: Option<String>,
    pub top_tracks: Vec<Track>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Playlist {
    pub is_owned: bool,
    pub title: String,
    pub duration_seconds: u32,
    pub tracks_count: u32,
    pub id: u32,
    pub cover_art: Option<String>,
    pub tracks: BTreeMap<u32, Track>,
}
