use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gstreamer as gst;
use gstreamer_play as gst_play;

use crate::ui::text;

#[derive(Clone, Debug)]
pub struct TrackRowData {
    pub uri: String,
    pub display: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleResult {
    /// Nothing to play (empty library / no selection and cache empty)
    NoTrack,

    /// We are now playing; includes which index is active.
    NowPlaying(usize),

    /// We are now paused; includes which index is active.
    NowPaused(usize),
}

pub struct PlaybackController {
    player: gst_play::Play,

    track_cache: Rc<RefCell<Vec<TrackRowData>>>,
    current_index: Rc<Cell<Option<usize>>>,
    is_playing: Rc<Cell<bool>>,
}

impl PlaybackController {
    pub fn new(
        track_cache: Rc<RefCell<Vec<TrackRowData>>>,
        current_index: Rc<Cell<Option<usize>>>,
        is_playing: Rc<Cell<bool>>,
    ) -> Self {
        gst::init().expect(text::GST_INIT_FAILED);
        let player = gst_play::Play::new(None::<gst_play::PlayVideoRenderer>);

        Self {
            player,
            track_cache,
            current_index,
            is_playing,
        }
    }

    /// Start playback from a specific index.
    /// Returns the display string to show in the UI.
    pub fn play_from_index(&self, index: usize) -> Option<String> {
        let cache = self.track_cache.borrow();
        if index >= cache.len() {
            return None;
        }
        let data = cache[index].clone();
        drop(cache);

        self.current_index.set(Some(index));
        self.is_playing.set(true);

        self.player.set_uri(Some(&data.uri));
        self.player.play();

        Some(data.display)
    }

    pub fn next_track(&self) -> Option<String> {
        let len = self.track_cache.borrow().len();
        if len == 0 {
            self.is_playing.set(false);
            return None;
        }

        let cur = self.current_index.get().unwrap_or(0);
        let next = (cur + 1) % len;

        self.play_from_index(next)
    }

    pub fn prev_track(&self) -> Option<String> {
        let len = self.track_cache.borrow().len();
        if len == 0 {
            self.is_playing.set(false);
            return None;
        }

        let cur = self.current_index.get().unwrap_or(0);
        let prev = if cur == 0 { len - 1 } else { cur - 1 };

        self.play_from_index(prev)
    }

    // Toggle play/pause. If nothing selected, attempts to start at index 0.
    pub fn toggle_play_pause(&self) -> ToggleResult {
        let idx_opt = self.current_index.get();

        // If nothing selected, try to start from the first track.
        if idx_opt.is_none() {
            let cache = self.track_cache.borrow();
            if cache.is_empty() {
                self.is_playing.set(false);
                return ToggleResult::NoTrack;
            }
            drop(cache);

            match self.play_from_index(0) {
                Some(_) => return ToggleResult::NowPlaying(0),
                None => return ToggleResult::NoTrack,
            }
        }

        let idx = idx_opt.unwrap();

        if self.is_playing.get() {
            self.player.pause();
            self.is_playing.set(false);
            ToggleResult::NowPaused(idx)
        } else {
            self.player.play();
            self.is_playing.set(true);
            ToggleResult::NowPlaying(idx)
        }
    }

    pub fn stop(&self) {
        self.player.stop();
        self.is_playing.set(false);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.get()
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index.get()
    }

    pub fn set_volume(&self, volume_0_to_1: f64) {
        // gst-play volume is a simple double; clamp for safety
        let v = volume_0_to_1.clamp(0.0, 1.0);
        self.player.set_volume(v);
    }

    pub fn seek_to_seconds(&self, seconds: f64) -> bool {
        if seconds.is_nan() || seconds.is_infinite() {
            return false;
        }

        let secs = if seconds < 0.0 { 0.0 } else { seconds };
        let ns = (secs * 1_000_000_000.0) as u64;

        self.player.seek(gst::ClockTime::from_nseconds(ns));
        true
    }
    pub fn duration_seconds(&self) -> Option<f64> {
        self.player
            .duration()
            .map(|d| d.nseconds() as f64 / 1_000_000_000.0)
    }
    pub fn position_seconds(&self) -> Option<f64> {
        self.player
            .position()
            .map(|p| p.nseconds() as f64 / 1_000_000_000.0)
    }
}
