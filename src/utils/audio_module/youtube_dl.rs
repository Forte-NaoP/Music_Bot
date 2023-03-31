use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Read},
    process::{Stdio, Command}, ptr::null,
    path::Path,
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

// group 5 is the video id;

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

const YTDL_COMMON_ARGS: [&str; 9] = [
    "-j",
    "--no-simulate",
    "-f",
    "webm[abr>0]/bestaudio/best",
    "-R",
    "infinite",
    "--no-playlist",
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

    let output_path = format!("{}{}", TMP_FORLDER, url.as_ref());
    let output = Path::new(&output_path);
    let value = if output.exists() && output.is_file() {
        // 파일이 있으면 메타데이터만 다운로드
        println!("file exists");
        _ytdl_optioned(&["--simulate", url.as_ref()]).await
    } else {
        // 파일이 없으면 다운로드
        println!("file not exists");
        _ytdl_optioned(&["--no-simulate", url.as_ref(), "-o", output_path.as_ref()]).await
    };

    let mut ffmpeg = Command::new("ffmpeg")
        .args(&["-ss", start.as_ref()])
        .args(&["-i", output_path.as_ref()])
        .args(&["-t", duration.as_ref()])
        .args(&FFMPEG_ARGS)
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let metadata = Metadata::from_ytdl_output(value?);
    //ffmpeg.wait().unwrap();
    Ok(Input::new(
        true,
        children_to_reader::<f32>(vec![ffmpeg]),
        Codec::FloatPcm,
        Container::Raw,
        Some(metadata),
    ))

}

async fn _ytdl_optioned(args: &[&str]) -> Result<Value> {
    let mut youtube_dl = Command::new(YOUTUBE_DL_COMMAND)
        .args(&YTDL_COMMON_ARGS)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let stderr = youtube_dl.stdout.take();
    let value = task::spawn_blocking(move || {
        let mut s = stderr.unwrap();
        let out: Result<Value> = {
            let mut o_vec = vec![];
            let mut serde_read = BufReader::new(s.by_ref());
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

    value
}