use serde_json::Value;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Cursor},
    process::{Stdio, Command},
    path::Path,
    mem,
};
use tokio::{
    process::{
        Command as TokioCommand,
    },
    task,
    io::{
        AsyncWriteExt,
    },
    fs::File as TokioFile,
};
use songbird::input::{
    Reader,
    error::{Error, Result},
    codec::OpusDecoderState, error::DcaError,
    Codec,
    Container,
    Input,
    Metadata as SongbirdMetadata,
};
use ogg::PacketReader;

use crate::utils::audio_module::metadata::Metadata;
use crate::utils::audio_module::dca;

const YOUTUBE_DL_COMMAND: &str = "yt-dlp";
const YTDL_COMMON_ARGS: [&str; 11] = [
    "-j",
    "--no-simulate",
    "-f",
    "webm[abr>0]/bestaudio/best",
    "-R",
    "infinite",
    "--no-playlist",
    "--ignore-config",
    "--no-warnings",
    "-o",
    "-"
];

const FFMPEG_DL_COMMAND: &str = "ffmpeg";
const FFMPEG_ARGS: [&str; 10] = [
    "-ac",
    "2",
    "-ar",
    "48000",
    "-ab",
    "64000",
    "-acodec",
    "libopus",
    "-f",
    "opus"
];

const TMP_FORLDER: &str = "./tmp/";

pub async fn ytdl(url: impl AsRef<str>) -> Result<Input> {
    ytdl_optioned(url, 0, 0).await
}

pub async fn ytdl_optioned(url: impl AsRef<str>, mut start: u64, mut duration: u64) -> Result<Input> {

    let audio_path = format!("{}{}.ogg", TMP_FORLDER, url.as_ref());
    let json_path = format!("{}{}.json", TMP_FORLDER, url.as_ref());

    let (audio, json) = (Path::new(&audio_path), Path::new(&json_path));
    
    let value = if audio.exists() && json.exists() {
        // 파일이 있으면 파일에서 메타데이터 읽어옴
        _metadata_from_file(json_path.to_owned()).await.unwrap()
    } else {
        // 파일이 없으면 ytdl로 다운받고 메타데이터 읽어옴
        _metadata_from_ytdl(url.as_ref().to_owned(), audio_path.clone(), json_path).await.unwrap()
    };

    let metadata = value.clone();
    let songbird_metadata = into_songbird_metadata(value);

    if duration == 0 {
        duration = songbird_metadata.duration.unwrap().as_secs();
    }
    if start >= songbird_metadata.duration.unwrap().as_secs() {
        start = 0;
    }
    if start + duration > songbird_metadata.duration.unwrap().as_secs() {
        duration = songbird_metadata.duration.unwrap().as_secs() - start;
    }
    
    let mut ffmpeg = Command::new(FFMPEG_DL_COMMAND)
        .args(&["-ss", start.to_string().as_str()])
        .args(&["-i", audio_path.as_str()])
        .args(&["-t", duration.to_string().as_str()])
        .args(&FFMPEG_ARGS)
        .arg("pipe:1")
        .stdout(Stdio::piped())
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut stdout = ffmpeg.stdout.take().unwrap();
    let mut o_vec = vec![];
    stdout.read_to_end(&mut o_vec).unwrap();
    ffmpeg.wait().unwrap();
    
    let mut cursor = Cursor::new(o_vec);
    let mut dca_input = dca::DcaWrapper::new(metadata);

    let mut reader = PacketReader::new(cursor);
    let mut skip = 2;
    while let Ok(packet) = reader.read_packet() {
        if let None = packet {
            break;
        } else if skip > 0 {
            skip -= 1;
            continue;
        }
        let packet = packet.unwrap();

        dca_input.write_audio_data(packet.data.as_slice());
    }

    Ok(Input::new(
        true,
        Reader::from_memory(dca_input.raw()),
        Codec::Opus(OpusDecoderState::new().map_err(DcaError::Opus)?),
        Container::Dca {
            first_frame: 0 //(header_size as usize) + mem::size_of::<i32>() + signature.len(),
        },
        Some(songbird_metadata),
    ))

}

async fn _metadata_from_ytdl(url: String, audio_path: String, json_path: String) -> Result<Metadata> {
    let mut youtube_dl = Command::new(YOUTUBE_DL_COMMAND)
        .args(&YTDL_COMMON_ARGS)
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let ytdl_stdout = youtube_dl.stdout.take().unwrap();
    let mut ytdl_stderr = youtube_dl.stderr.take().unwrap();

    let mut ffmpeg = Command::new(FFMPEG_DL_COMMAND)
        .args(&["-i", "pipe:0"])
        .args(&FFMPEG_ARGS)
        .arg(audio_path)
        .stdin(ytdl_stdout)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let mut data = vec![];
    let out_size = ytdl_stderr.read_to_end(&mut data);

    let metadata: Result<Value> = if let Ok(_) = out_size {
        serde_json::from_slice(&data).map_err(|err| Error::Json {
            error: err,
            parsed_text: std::str::from_utf8(&data).unwrap_or_default().to_string(),
        })
    } else {
        Result::Err(Error::Metadata)
    };
    
    let metadata = Metadata::from_ytdl_output(metadata.expect("ssibal"));
    let metadata_clone = metadata.clone();

    let handle = tokio::task::spawn(async move {
        let mut json = TokioFile::create(json_path).await.unwrap();
        let data = serde_json::to_string(&metadata_clone).unwrap();
        json.write_all(data.as_bytes()).await.unwrap();
        json.flush().await.unwrap();
    });

    youtube_dl.wait().unwrap();
    ffmpeg.wait().unwrap();
    handle.await.unwrap();

    Ok(metadata)
}

async fn _metadata_from_file(url: String) -> Result<Metadata> {
    let mut json = File::open(url).expect("_metadata_from_file open error");

    let mut metadata_buf = vec![];
    json.read_to_end(&mut metadata_buf).expect("json.read_to_end");
    let metadata = serde_json::from_slice(&metadata_buf).unwrap();

    Ok(metadata)
}

fn into_songbird_metadata(metadata: Metadata) -> SongbirdMetadata {
    SongbirdMetadata {
        track: metadata.track,
        artist: metadata.artist,
        date: metadata.date,
        channels: metadata.channels,
        channel: metadata.channel,
        start_time: metadata.start_time,
        duration: metadata.duration,
        sample_rate: metadata.sample_rate,
        source_url: metadata.source_url,
        title: metadata.title,
        thumbnail: metadata.thumbnail
    }
}