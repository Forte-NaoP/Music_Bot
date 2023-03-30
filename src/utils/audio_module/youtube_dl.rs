use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Read},
    process::{Stdio, Command}, ptr::null,
};
use tokio::{
    process::{
        Command as TokioCommand,
    },
    task,
    io::{
        BufReader as TokioBufReader,
        AsyncBufReadExt,
        AsyncRead,
        AsyncWrite,
        Stderr,
        Stdout,
        Stdin,
    }
};
use songbird::input::{
    children_to_reader,
    error::{Error, Result},
    Codec,
    Container,
    Input,
    Metadata, ffmpeg_optioned,
};

use crate::utils::url_checker::YOUTUBE_PREFIX;

const YOUTUBE_DL_COMMAND: &str = "yt-dlp";
const YTDL_PRE_ARGS: [&str; 10] = [
    "-j",
    "--no-simulate",
    "-f",
    "webm[abr>0]/bestaudio/best",
    "-R",
    "infinite",
    "--no-playlist",
    "--no-overwrites",
    "--ignore-config",
    "--no-warnings",
];

const FFMPEG_ARGS: [&str; 9] = [
    "-f",
    "s16le",
    "-ac",
    "2",
    "-ar",
    "48000",
    "-acodec",
    "pcm_f32le",
    "-",
];

const TMP_FORLDER: &str = "./tmp/";

pub async fn ytdl_optioned(url: impl AsRef<str>, start: String, duration: String) -> Result<Input> {

    let mut youtube_dl = Command::new(YOUTUBE_DL_COMMAND)
        .args(&YTDL_PRE_ARGS)
        .arg(format!("{}{}", YOUTUBE_PREFIX, url.as_ref()))
        .arg("-o")
        .arg(format!("{}{}", TMP_FORLDER, url.as_ref()))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let stderr = youtube_dl.stdout.take();
    let value = task::spawn_blocking(move || {
        let mut s = stderr.unwrap();
        let out: Result<Value> = {
            let mut o_vec = vec![];
            let mut serde_read = BufReader::new(s.by_ref());
            // Newline...
            if let Ok(len) = serde_read.read_until(0xA, &mut o_vec) {
                serde_json::from_slice(&o_vec[..len]).map_err(|err| Error::Json {
                    error: err,
                    parsed_text: std::str::from_utf8(&o_vec).unwrap_or_default().to_string(),
                })
            } else {
                Result::Err(Error::Metadata)
            }
        };

        out
    })
    .await
    .map_err(|_| Error::Metadata)?;
    
    //wait 안하면 파일이 저장되기 전에 ffmpeg가 실행되서 파일을 못 읽어들임
    youtube_dl.wait().unwrap();

    let mut ffmpeg = Command::new("ffmpeg")
        .arg("-ss")
        .arg(start.to_string())
        .arg("-i")
        .arg(format!("{}{}", TMP_FORLDER, url.as_ref()))
        .arg("-t")
        .arg(duration.to_string())
        .args(&FFMPEG_ARGS)
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let metadata = Metadata::from_ytdl_output(value?);

    Ok(Input::new(
        true,
        children_to_reader::<f32>(vec![ffmpeg]),
        Codec::FloatPcm,
        Container::Raw,
        Some(metadata),
    ))

}