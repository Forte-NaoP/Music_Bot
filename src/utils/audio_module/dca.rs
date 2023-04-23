use ogg::PacketReader;
use serde_json::Value;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::utils::audio_module::metadata::Metadata;

static DCA_MAGIC_BYTES: &'static [u8; 4] = b"DCA1";

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct DcaMetadata {
    pub(crate) dca: Dca,
    pub(crate) opus: Opus,
    pub(crate) info: Option<Info>,
    pub(crate) origin: Option<Origin>,
    pub(crate) extra: Option<Extra>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Dca {
    pub(crate) version: u64,
    pub(crate) tool: Tool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Tool {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) url: String,
    pub(crate) author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Opus {
    pub(crate) mode: String,
    pub(crate) sample_rate: u32,
    pub(crate) frame_size: u64,
    pub(crate) abr: u64,
    pub(crate) vbr: bool,
    pub(crate) channels: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Info {
    pub(crate) title: Option<String>,
    pub(crate) artist: Option<String>,
    pub(crate) album: Option<String>,
    pub(crate) genre: Option<String>,
    pub(crate) cover: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Origin {
    pub(crate) source: Option<String>,
    pub(crate) abr: Option<u64>,
    pub(crate) channels: Option<u8>,
    pub(crate) encoding: Option<String>,
    pub(crate) url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Extra {}

pub struct DcaWrapper {
    metadata: Metadata,
    raw: Vec<u8>,
}

impl DcaWrapper {
    pub fn new(metadata: Metadata) -> Self {
        DcaWrapper { metadata, raw: vec![] }
    }

    pub fn raw(&self) -> Vec<u8> {
        self.raw.clone()
    }

    pub fn write_audio_data(&mut self, audio_data: &[u8]) {
        let data_len = audio_data.len() as i16;
        self.raw.extend_from_slice(&data_len.to_le_bytes());
        self.raw.extend_from_slice(audio_data);
    }

    pub fn write_dca_header(&mut self) {
        let dca_header = DcaMetadata {
            dca: DcaWrapper::dca(),
            opus: DcaWrapper::opus(),
            info: DcaWrapper::info(&self),
            origin: DcaWrapper::origin(&self),
            extra: DcaWrapper::extra(),
        };
        self.raw.extend_from_slice(DCA_MAGIC_BYTES);
        let dca_header = serde_json::to_string(&dca_header).unwrap();
        let dca_header_len = dca_header.as_bytes().len() as i32;
        self.raw.extend_from_slice(&dca_header_len.to_le_bytes());
        self.raw.extend_from_slice(dca_header.as_bytes());
    }

    fn dca() -> Dca {
        Dca {
            version: 1,
            tool: Tool {
                name: "dca-rs".to_owned(),
                version: "1.0.0".to_owned(),
                url: "https://github.com/Forte-NaoP/dca-rs".to_owned(),
                author: "Forte-NaoP".to_owned(),
            }
        }
    }

    fn opus() -> Opus {
        Opus {
            mode: String::from("voip"),
            sample_rate: 48000,
            frame_size: 960,
            abr: 64000,
            vbr: true,
            channels: 2,
        }
    }

    fn info(&self) -> Option<Info> {
        Some(Info {
            title: Some(self.metadata.title.as_ref().unwrap().to_owned()),
            artist: Some(self.metadata.artist.as_ref().unwrap().to_owned()),
            album: None,
            genre: None,
            cover: None,
        })
    }

    fn origin(&self) -> Option<Origin> {
        Some(Origin {
            source: Some("file".to_owned()),
            abr: Some(64000),
            channels: Some(2),
            encoding: Some("Ogg".to_owned()),
            url: Some(self.metadata.source_url.as_ref().unwrap().to_string()),
        })
    }

    fn extra() -> Option<Extra> {
        Some(Extra {})
    }

}
