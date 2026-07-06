use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
pub use rodio::source::SeekError;
use rodio::Source;
use std::{num::NonZero, sync::OnceLock};
use std::{sync::Mutex, time::Duration};
use stream_download::{
    http::{HttpStream, HttpStreamError},
    storage::{bounded::BoundedStorageProvider, memory::MemoryStorageProvider},
    StreamDownload, StreamInitializationError,
};

#[derive(Debug, thiserror::Error)]
pub enum InitPlayerError {
    #[error("Device sink error: {0}")]
    DeviceSinkError(#[from] rodio::DeviceSinkError),
    #[error("Set static variable failed: {0}")]
    SetStatocVariableFailed(String),
}

#[flutter_rust_bridge::frb(sync)]
pub fn init_player() -> Result<(), InitPlayerError> {
    let sink = rodio::DeviceSinkBuilder::open_default_sink()?;
    let player = rodio::Player::connect_new(&sink.mixer());
    SINK.set(sink)
        .map_err(|_| InitPlayerError::SetStatocVariableFailed("sink".into()))?;
    PLAYER
        .set(player)
        .map_err(|_| InitPlayerError::SetStatocVariableFailed("player".into()))?;
    Ok(())
}

static SINK: OnceLock<rodio::MixerDeviceSink> = OnceLock::new();
static PLAYER: OnceLock<rodio::Player> = OnceLock::new();
static TOTAL_DURATION: Mutex<Option<SecondDuration>> = Mutex::new(None);

pub struct SecondDuration {
    pub inner: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadAudioSourceError {
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] std::string::ParseError),
    #[error("HTTP stream error: {0}")]
    HttpStreamError(#[from] HttpStreamError<reqwest::Client>),
    #[error("Stream initialization error: {0}")]
    StreamInitializationError(#[from] StreamInitializationError<HttpStream<reqwest::Client>>),
    #[error("Decoder error: {0}")]
    DecoderError(#[from] rodio::decoder::DecoderError),
    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

#[flutter_rust_bridge::frb]
pub async fn load_audio_source_and_play(
    url: String,
    jwt_value: String,
) -> Result<(), LoadAudioSourceError> {
    let mut headers = HeaderMap::new();

    let jwt = format!("Bearer {}", jwt_value);
    let mut auth_value = HeaderValue::from_str(&jwt)?;
    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    let reqwest_client = Client::builder().default_headers(headers).build()?;
    let url = url.parse().unwrap();
    let stream = HttpStream::new(reqwest_client, url).await?;

    let reader = StreamDownload::from_stream(
        stream,
        BoundedStorageProvider::new(
            MemoryStorageProvider::default(),
            NonZero::new(1024 * 1024).unwrap(),
        ),
        stream_download::Settings::default(),
    )
    .await?;

    tokio::task::spawn_blocking(move || -> Result<(), rodio::decoder::DecoderError> {
        let player = PLAYER.get().unwrap();
        let source = rodio::Decoder::new(std::io::BufReader::new(reader))?;
        let mut total_duration = TOTAL_DURATION.lock().unwrap();
        *total_duration = source.total_duration().map(|x| SecondDuration {
            inner: x.as_secs_f64(),
        });
        player.stop();
        player.append(source);
        player.play();
        Ok(())
    })
    .await??;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum PlayAudioError {
    #[error("Decoder error: {0}")]
    DecoderError(#[from] rodio::decoder::DecoderError),
}

/// Retrun value means is_paused
#[flutter_rust_bridge::frb(sync)]
pub fn play_or_pause_audio() -> Result<bool, PlayAudioError> {
    let player = PLAYER.get().unwrap();
    if player.is_paused() {
        player.play();
        return Ok(true);
    } else {
        player.pause();
        return Ok(false);
    }
}

#[flutter_rust_bridge::frb(sync)]
pub fn stop_audio() -> Result<(), PlayAudioError> {
    let player = PLAYER.get().unwrap();
    player.stop();
    Ok(())
}

#[flutter_rust_bridge::frb(sync)]
pub fn get_total_duration() -> Option<f64> {
    let total_duration = TOTAL_DURATION.lock().unwrap();
    total_duration.as_ref().map(|x| x.inner)
}

#[flutter_rust_bridge::frb(sync)]
pub fn get_position() -> f64 {
    let player = PLAYER.get().unwrap();
    player.get_pos().as_secs_f64()
}

#[flutter_rust_bridge::frb(sync)]
pub fn try_seek(position: f64) -> Result<(), SeekError> {
    let player = PLAYER.get().unwrap();
    player.play();
    player.try_seek(Duration::from_secs_f64(position))?;
    Ok(())
}
