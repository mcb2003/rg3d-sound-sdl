# rg3d-sound-sdl

Use rg3d-sound with SDL's audio subsystem

This crate allows you to use [SDL2's][sdl2] audio backend to output rendered audio data from
[`rg3d_sound`]. This provides maximum portability between operating systems as SDL audio works
almost everywhere SDL does, and it will also take advantage of newer audio interfaces like
Pulseaudio and PipeWire on Linux.

# Example

```rust
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
```

# License

MIT License

Copyright (c) 2022 Michael Connor Buchan <mikey@blindcomputing.org>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
