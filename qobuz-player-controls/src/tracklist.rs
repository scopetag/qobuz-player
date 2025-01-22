use crate::service::{Album, Playlist, Track, TrackStatus};
use std::collections::BTreeMap;
use tracing::instrument;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum TrackListType {
    Album,
    Playlist,
    Track,
    #[default]
    Unknown,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Tracklist {
    pub queue: BTreeMap<u32, Track>,
    pub album: Option<Album>,
    pub playlist: Option<Playlist>,
    pub list_type: TrackListType,
}

impl Tracklist {
    #[instrument]
    pub fn new(queue: Option<&BTreeMap<u32, Track>>) -> Tracklist {
        Tracklist {
            queue: queue.unwrap_or(&BTreeMap::new()).clone(),
            album: None,
            playlist: None,
            list_type: TrackListType::Unknown,
        }
    }

    pub fn total(&self) -> u32 {
        if let Some(album) = &self.album {
            album.total_tracks
        } else if let Some(list) = &self.playlist {
            list.tracks_count
        } else {
            self.queue.len() as u32
        }
    }

    #[instrument(skip(self))]
    pub fn get_album(&self) -> Option<&Album> {
        if let Some(c) = self.current_track() {
            if let Some(album) = &c.album {
                Some(album)
            } else {
                self.album.as_ref()
            }
        } else {
            self.album.as_ref()
        }
    }

    #[instrument(skip(self))]
    pub fn get_playlist(&self) -> Option<&Playlist> {
        self.playlist.as_ref()
    }

    #[instrument(skip(self))]
    pub fn set_list_type(&mut self, list_type: TrackListType) {
        self.list_type = list_type;
    }

    #[instrument(skip(self))]
    pub fn list_type(&self) -> &TrackListType {
        &self.list_type
    }

    #[instrument(skip(self))]
    pub fn set_track_status(&mut self, position: u32, status: TrackStatus) {
        if let Some(track) = self.queue.get_mut(&position) {
            track.status = status;
        }
    }

    #[instrument(skip(self))]
    pub fn all_tracks(&self) -> Vec<&Track> {
        self.queue.values().collect::<Vec<&Track>>()
    }

    #[instrument(skip(self))]
    pub fn unplayed_tracks(&self) -> Vec<&Track> {
        self.queue
            .iter()
            .filter_map(|t| {
                if t.1.status == TrackStatus::Unplayed {
                    Some(t.1)
                } else {
                    None
                }
            })
            .collect::<Vec<&Track>>()
    }

    #[instrument(skip(self))]
    pub fn played_tracks(&self) -> Vec<&Track> {
        self.queue
            .iter()
            .filter_map(|t| {
                if t.1.status == TrackStatus::Played {
                    Some(t.1)
                } else {
                    None
                }
            })
            .collect::<Vec<&Track>>()
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.queue
            .values()
            .find(|&track| track.status == TrackStatus::Playing)
    }

    pub fn cursive_list(&self) -> Vec<(&str, i32)> {
        self.queue
            .values()
            .map(|i| (i.title.as_str(), i.id as i32))
            .collect::<Vec<(&str, i32)>>()
    }
}
