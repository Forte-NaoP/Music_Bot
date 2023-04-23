use serde_json::Value;
use std::time::Duration;
use serde::{Serialize, Deserialize};

pub const SAMPLE_RATE: u32 = 48000;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Metadata {
    /// The track of this stream.
    pub track: Option<String>,
    /// The main artist of this stream.
    pub artist: Option<String>,
    /// The date of creation of this stream.
    pub date: Option<String>,

    /// The number of audio channels in this stream.
    ///
    /// Any number `>= 2` is treated as stereo.
    pub channels: Option<u8>,
    /// The YouTube channel of this stream.
    pub channel: Option<String>,
    /// The time at which the first true sample is played back.
    ///
    /// This occurs as an artefact of coder delay.
    pub start_time: Option<Duration>,
    /// The reported duration of this stream.
    pub duration: Option<Duration>,
    /// The sample rate of this stream.
    pub sample_rate: Option<u32>,
    /// The source url of this stream.
    pub source_url: Option<String>,
    /// The YouTube title of this stream.
    pub title: Option<String>,
    /// The thumbnail url of this stream.
    pub thumbnail: Option<String>,
}

impl Metadata {
    /// Use `youtube-dl`'s JSON output for metadata for an online resource.
    pub fn from_ytdl_output(value: Value) -> Self {
        let obj = value.as_object();

        let track = obj
            .and_then(|m| m.get("track"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let true_artist = obj
            .and_then(|m| m.get("artist"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let artist = true_artist.or_else(|| {
            obj.and_then(|m| m.get("uploader"))
                .and_then(Value::as_str)
                .map(str::to_string)
        });

        let r_date = obj
            .and_then(|m| m.get("release_date"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let date = r_date.or_else(|| {
            obj.and_then(|m| m.get("upload_date"))
                .and_then(Value::as_str)
                .map(str::to_string)
        });

        let channel = obj
            .and_then(|m| m.get("channel"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let duration = obj
            .and_then(|m| m.get("duration"))
            .and_then(Value::as_f64)
            .map(Duration::from_secs_f64);

        let source_url = obj
            .and_then(|m| m.get("webpage_url"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let title = obj
            .and_then(|m| m.get("title"))
            .and_then(Value::as_str)
            .map(str::to_string);

        let thumbnail = obj
            .and_then(|m| m.get("thumbnail"))
            .and_then(Value::as_str)
            .map(str::to_string);

        Self {
            track,
            artist,
            date,

            channels: Some(2),
            channel,
            duration,
            sample_rate: Some(SAMPLE_RATE),
            source_url,
            title,
            thumbnail,

            ..Default::default()
        }
    }
}
