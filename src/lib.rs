//! Use rg3d-sound with SDL's audio subsystem
//!
//! This crate allows you to use [SDL2's][sdl2] audio backend to output rendered audio data from
//! [`rg3d_sound`]. This provides maximum portability between operating systems as SDL audio works
//! almost everywhere SDL does, and it will also take advantage of newer audio interfaces like
//! Pulseaudio and PipeWire on Linux.
//! # Example
//! ```no_run
//! use std::{fs::File, io::BufReader, thread, time::Duration};
//! # use std::error::Error;
//!
//! use rg3d_sound::{
//! buffer::{DataSource, SoundBufferResource},
//! context::SoundContext,
//! source::{generic::GenericSourceBuilder, Status},
//! };
//!
//!# fn main() -> Result<(), Box<dyn Error>> {
//! let sdl = sdl2::init()?;
//! let audio = sdl.audio()?;
//! let (engine, device) = rg3d_sound_sdl::open(&audio, None)?;
//! device.resume();
//!
//! let ctx = SoundContext::new();
//! engine.lock().unwrap().add_context(ctx.clone());
//!
//! let sound_buffer = SoundBufferResource::new_generic(DataSource::File {
//! path: "ding.wav".into(),
//! data: BufReader::new(File::open("ding.wav")?),
//! })
//! .expect("Failed to create data source");
//!
//! let source = GenericSourceBuilder::new()
//! .with_buffer(sound_buffer)
//! .with_status(Status::Playing)
//! .build_source()?;
//!
//! ctx.state().add_source(source);
//!
//! thread::sleep(Duration::from_millis(1090));
//! # Ok(())
//! # }
//! ```

use std::sync::{Arc, Mutex};

use rg3d_sound::engine::SoundEngine;
use sdl2::audio::{AudioCallback, AudioDevice, AudioFormat, AudioSpecDesired};

/// Opens a new audio device.
///
/// On success, returns both the SDL [`AudioDevice`], and a handle to a
/// [`SoundEngine`] which will drive the device. On error, returns the SDL error.
/// # Example
/// ```no_run
/// let sdl = sdl2::init().unwrap();
/// let audio = sdl.audio().unwrap();
/// let (engine, device) = rg3d_sound_sdl::open(&audio, None).unwrap();
/// device.resume();
/// ```
pub fn open<'a>(
    subsystem: &sdl2::AudioSubsystem,
    device: impl Into<Option<&'a str>>,
) -> Result<(Arc<Mutex<SoundEngine>>, AudioDevice<Callback>), String> {
    let desired = desired_spec();
    let engine = SoundEngine::without_device();
    let callback_engine = Arc::clone(&engine);

    subsystem
        .open_playback(device, &desired, |obtained| {
            assert_eq!(
                obtained.freq as u32,
                rg3d_sound::context::SAMPLE_RATE,
                "Invalid sample rate"
            );
            assert_eq!(obtained.channels, 2, "Invalid number of channels");
            assert_eq!(
                obtained.format,
                AudioFormat::f32_sys(),
                "Invalid sample format"
            );
            assert_eq!(
                obtained.samples as usize,
                SoundEngine::render_buffer_len(),
                "Invalid buffer size"
            );
            Callback::new(callback_engine)
        })
        .map(|dev| (engine, dev))
}

/// Obtain the desired SDL audio parameters for use with `rg3d_sound`. This is used internally by
/// [`open`] to configure the playback device.
/// # Panics
/// This function will panic if the returned buffer size from [`SoundEngine::render_buffer_len`] is
/// too large for SDL (I.E. buffer_size > u16::MAX).
///
/// This crate also staticly asserts that [`SAMPLE_RATE`][rg3d_sound::context::SAMPLE_RATE] <=
/// `i32::MAX`.
/// # Example
/// ```
/// let desired = rg3d_sound_sdl::desired_spec();
/// assert_eq!(desired.freq, Some(44_100));
/// assert_eq!(desired.channels, Some(2));
/// ```
pub fn desired_spec() -> AudioSpecDesired {
    let samples = SoundEngine::render_buffer_len()
        .try_into()
        .expect("Audio buffer too large");
    AudioSpecDesired {
        freq: Some(rg3d_sound::context::SAMPLE_RATE as _),
        channels: Some(2),
        samples: Some(samples),
    }
}

/// An [`AudioCallback`] used to feed the SDL audio device with rendered audio from a
/// [`SoundEngine`]
pub struct Callback {
    engine: Arc<Mutex<SoundEngine>>,
}

impl Callback {
    /// Create a new `Callback` from an existing [`SoundEngine`]. The engine must be opened with
    /// [`SoundEngine::without_device`] so that the manual rendering functions can be used.
    pub fn new(engine: Arc<Mutex<SoundEngine>>) -> Self {
        Self { engine }
    }
}

impl AudioCallback for Callback {
    type Channel = f32;

    fn callback(&mut self, buf: &mut [Self::Channel]) {
        let buf = to_tuple_slice(buf);
        let mut engine = self.engine.lock().unwrap();
        engine.render(buf);
    }
}

/// Converts a slice of [`f32`] values, of even length, to a slice of `(f32, f32)` tuples. The
/// returned slice will be half the length of the input slice.
/// # Panics
/// This function will panic if the input slice has an odd number of elements.
///
/// This crate also staticly asserts that the alignment and size of `(f32, f32)` and `[f32; 2]` are
/// identical.
pub fn to_tuple_slice(slice: &mut [f32]) -> &mut [(f32, f32)] {
    let ptr = slice.as_mut_ptr();
    let len = slice.len();
    debug_assert!(len % 2 == 0);
    unsafe { std::slice::from_raw_parts_mut(ptr.cast(), len / 2) }
}

static_assertions::assert_eq_align!((f32, f32), [f32; 2]);
static_assertions::assert_eq_size!((f32, f32), [f32; 2]);

static_assertions::const_assert!(rg3d_sound::context::SAMPLE_RATE <= i32::MAX as u32);
