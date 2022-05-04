use std::sync::{Arc, Mutex};

use rg3d_sound::engine::SoundEngine;
use sdl2::audio::{AudioCallback, AudioDevice, AudioFormat, AudioSpecDesired};

pub fn open<'a, D>(
    subsystem: &sdl2::AudioSubsystem,
    device: D,
) -> Result<(Arc<Mutex<SoundEngine>>, AudioDevice<Callback>), String>
where
    D: Into<Option<&'a str>>,
{
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

pub struct Callback {
    engine: Arc<Mutex<SoundEngine>>,
}

impl Callback {
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

fn to_tuple_slice(slice: &mut [f32]) -> &mut [(f32, f32)] {
    let ptr = slice.as_mut_ptr();
    let len = slice.len();
    debug_assert!(len % 2 == 0);
    unsafe { std::slice::from_raw_parts_mut(ptr.cast(), len / 2) }
}

static_assertions::assert_eq_align!((f32, f32), [f32; 2]);
static_assertions::assert_eq_size!((f32, f32), [f32; 2]);

static_assertions::const_assert!(rg3d_sound::context::SAMPLE_RATE <= i32::MAX as u32);
