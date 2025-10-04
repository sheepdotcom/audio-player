use std::{fs::File, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use symphonia::{core::{codecs::{Decoder, DecoderOptions}, formats::{FormatOptions, FormatReader, Track}, io::{MediaSourceStream, MediaSourceStreamOptions}, meta::MetadataOptions, probe::Hint}, default::{get_codecs, get_probe}};

#[cfg(not(target_os = "linux"))]
use cpal::{traits::{DeviceTrait, StreamTrait}, Device, OutputCallbackInfo};

#[derive(Serialize, Deserialize)]
pub struct Song {
    pub name: String,
    pub author: String,
    pub album: Option<String>, // for game OSTs it's gonna be the name of the game or something you know, like BIAST OST
    pub youtube_link: Option<String>,
    pub length: String, // ok change this type man this aint gonna work right now
    pub song_type: SongType,
}

impl Song {
    // STOP THINKING AHEAD JUST DO THE CPAL STUFF FIRST
}

#[derive(Serialize, Deserialize)]
pub enum SongType {
    LOCAL(LocalSong),
    ONLINE(OnlineSong),
}

#[derive(Serialize, Deserialize)]
pub struct LocalSong {
    pub file_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct OnlineSong {
    pub link: String,
}

// TODO: move this somewhere idk no way this is staying in main.rs
#[derive(Clone, Default)]
pub struct AppState {
    #[cfg(target_os = "linux")]
    pub connection: Arc<Mutex<Option<psimple::Simple>>>,

    #[cfg(not(target_os = "linux"))]
    pub device: Arc<Mutex<Option<Device>>>,

    pub cursor: Arc<Mutex<usize>>,
    pub exit: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(not(target_os = "linux"))]
fn load_audio_file<P: AsRef<Path>>(path: P) -> Result<(Vec<f32>, u32)> {
    let (mut format, track, mut decoder) = decode_audio_file(path)?;

    let mut samples = Vec::new();

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track.id {
            continue;
        }

        let decoded_packet = decoder.decode(&packet)
            .map_err(|err| eyre!("packet failed to decode: {err}"))?;

        let mut buffer: SampleBuffer<f32> = SampleBuffer::new(decoded_packet.capacity() as u64, *decoded_packet.spec());
        buffer.copy_interleaved_ref(decoded_packet);

        samples.extend_from_slice(buffer.samples());
    }

    Ok((samples, track.codec_params.sample_rate.unwrap_or(44100)))
}

#[cfg(target_os = "linux")]
fn load_audio_file<P: AsRef<Path>>(path: P) -> Result<(Vec<u8>, u32)> {
    let (mut format, track, mut decoder) = decode_audio_file(path)?;

    let mut samples = Vec::new();

    while let Ok(packet) = format.next_packet() {
        use symphonia::core::audio::RawSampleBuffer;

        if packet.track_id() != track.id {
            continue;
        }

        let decoded_packet = decoder.decode(&packet)
            .map_err(|err| eyre!("packet failed to decode: {err}"))?;

        let mut buffer: RawSampleBuffer<f32> = RawSampleBuffer::new(decoded_packet.capacity() as u64, *decoded_packet.spec());
        buffer.copy_interleaved_ref(decoded_packet);

        samples.extend_from_slice(buffer.as_bytes());
    }

    Ok((samples, track.codec_params.sample_rate.unwrap_or(44100)))
}

#[allow(clippy::type_complexity)]
fn decode_audio_file<P: AsRef<Path>>(path: P) -> Result<(Box<dyn FormatReader>, Track, Box<dyn Decoder>)> {
    let probe = get_probe();

    let file = File::open(path)
        .map_err(|err| eyre!("failed to open file: {err}"))?;

    let mss = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());

    let hint = Hint::new(); // there are .with_extension and .mine_type hints, maybe later
    let format_opts = FormatOptions { enable_gapless: true, ..Default::default() };
    let metadata_opts = MetadataOptions::default();

    let probe_result = probe
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|err| eyre!("input format not supported: {err}"))?;

    let format = probe_result.format;
    let track = format.default_track()
        .ok_or_else(|| eyre!("no default track"))?
        .clone();

    let decoder_opts = DecoderOptions { verify: true };

    let decoder = get_codecs()
        .make(&track.codec_params, &decoder_opts)
        .map_err(|err| eyre!("input codec not supported: {err}"))?;

    Ok((format, track, decoder))
}

#[cfg(not(target_os = "linux"))]
pub fn setup_audio(state: &mut AppState) -> Result<()> {
    let host = cpal::default_host();

    let device = host.default_output_device()
        .ok_or_else(|| eyre!("No output device available, you shouldn't be using this if you got nothing that can output sound."))?;

    {
        let mut device_lock = state.device.lock()
            .unwrap_or_else(|err| err.into_inner());

        *device_lock = Some(device);
    }

    Ok(())
}

// this code here is based on symphonia-play, since it shows how to do proper linux support (since cpal is kinda broken with it)
#[cfg(target_os = "linux")]
pub fn setup_audio(state: &mut AppState) -> Result<()> {
    let spec = pulse::sample::Spec {
        format: pulse::sample::Format::FLOAT32NE,
        channels: 2,
        rate: 48_000,
    };

    let mut map = pulse::channelmap::Map::default();
    map.init_stereo();

    let connection = psimple::Simple::new(
        None,
        "audio_player yeah generic name",
        pulse::stream::Direction::Playback,
        None,
        "audio_player app wip thing yeah hi linux user",
        &spec,
        Some(&map),
        None,
    )?;

    {
        let mut connection_lock = state.connection.lock()
            .unwrap_or_else(|err| err.into_inner());

        *connection_lock = Some(connection);
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn play_audio(state: &mut AppState) -> Result<()> {
    // bind it to a variable or compiler error wow so great extra lines of code (because im handling poisoned locks now)
    let mut device_lock = state.device.lock()
        .unwrap_or_else(|err| err.into_inner());

    let device = device_lock.as_mut()
        .ok_or_else(|| eyre!("oops tried to play audio with no device!"))?;
    
    let config = device.default_output_config()?;

    let (pcm, sample_rate) = load_audio_file("/home/sheepdotcom/Music/forsaken/music/divadayo_chase_theme_compat.ogg")?;

    let pcm_clone = pcm.clone();
    let cursor_clone = state.cursor.clone();

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &OutputCallbackInfo| {
            let mut cursor = cursor_clone.lock()
                .unwrap_or_else(|err| err.into_inner());

            for frame in data.iter_mut() {
                *frame = pcm_clone[*cursor];
                *cursor += 1;
                if *cursor >= pcm_clone.len() {
                    *cursor = 0; // auto-loop for now, this is just a test
                }
            }
        },
        |err| eprintln!("stream error: {err}"),
        None
    ).map_err(|err| eyre!("failed to build stream: {err}"))?;

    stream.play()?;

    Ok(())
}

// this code here is based on symphonia-play, since it shows how to do proper linux support (since cpal is kinda broken with it)
#[cfg(target_os = "linux")]
pub fn play_audio(state: &mut AppState) -> Result<()> {
    // test audio whatever
    let (pcm, sample_rate) = load_audio_file("/home/sheepdotcom/Music/forsaken/music/divadayo_chase_theme_compat.ogg")?;

    eprintln!("pcm len: {}", pcm.len());

    let connection_clone = state.connection.clone();
    let pcm_clone = pcm.clone();
    let cursor_clone = state.cursor.clone();

    std::thread::spawn(move || -> Result<()> {
        let mut connection_lock = connection_clone.lock()
            .unwrap_or_else(|err| err.into_inner());

        let connection = connection_lock.as_mut()
            .ok_or_else(|| eyre!("oops tried to play audio with no device!"))?;

        loop { // forever get next buffer and write it :3
            let mut cursor = cursor_clone.lock()
                .unwrap_or_else(|err| err.into_inner());

            // check because outside code doesn't know the exact audio length so it could have gone out of bounds
            if *cursor >= pcm_clone.len() {
                *cursor %= pcm_clone.len();
            }

            let mut data = [0u8; 2048]; // 1024 frames, but 2 channels so double that

            for frame in data.iter_mut() {
                *frame = pcm_clone[*cursor];
                *cursor += 1;
                if *cursor >= pcm_clone.len() {
                    *cursor = 0; // loop audio
                }
            }

            // drop the lock on cursor so other parts of code can write to it between pulseaudio buffer writes
            drop(cursor);

            connection.write(&data)?;
        }
    });

    Ok(())
}
