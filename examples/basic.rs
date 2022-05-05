use std::{error::Error, fs::File, io::BufReader, thread, time::Duration};

use rg3d_sound::{
    buffer::{DataSource, SoundBufferResource},
    context::SoundContext,
    source::{generic::GenericSourceBuilder, Status},
};

fn main() -> Result<(), Box<dyn Error>> {
    let sdl = sdl2::init()?;
    let audio = sdl.audio()?;
    let (engine, device) = rg3d_sound_sdl::open(&audio, None)?;
    device.resume();

    let ctx = SoundContext::new();
    engine.lock().unwrap().add_context(ctx.clone());

    let sound_buffer = SoundBufferResource::new_generic(DataSource::File {
        path: "ding.wav".into(),
        data: BufReader::new(File::open("ding.wav")?),
    })
    .expect("Failed to create data source");

    let source = GenericSourceBuilder::new()
        .with_buffer(sound_buffer)
        .with_status(Status::Playing)
        .build_source()?;

    ctx.state().add_source(source);

    thread::sleep(Duration::from_millis(1090));
    Ok(())
}
