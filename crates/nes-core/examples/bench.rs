//! Mide la velocidad del core: cuántos frames por segundo emula sin ventana ni sonido.
//!
//! ```sh
//! cargo run -p nes-core --release --example bench                 # ROM de demostración
//! cargo run -p nes-core --release --example bench -- juego.nes    # una ROM propia
//! ```

use std::time::Instant;

use nes_core::{Emulator, demo_rom};

/// Frames que se emulan en cada medición (unos 50 segundos de juego).
const FRAMES: u32 = 3000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (name, rom) = match std::env::args_os().nth(1) {
        Some(path) => (path.to_string_lossy().into_owned(), std::fs::read(&path)?),
        None => ("demo.nes".to_owned(), demo_rom()),
    };

    let mut emulator = Emulator::new();
    emulator.load_rom(&name, &rom)?;
    let real_time = emulator.region().frame_rate();
    println!("ROM: {name} ({FRAMES} frames por medición)");

    let full = measure(|| emulator.run_frame().map_err(Into::into))?;
    report("Frame completo (vídeo y audio)", full, real_time);

    let silent = measure(|| emulator.run_frame_silent().map_err(Into::into))?;
    report("Frame sin audio", silent, real_time);

    let skipped = measure(|| emulator.skip_frame().map_err(Into::into))?;
    report("Frame omitido (sin vídeo ni audio)", skipped, real_time);
    Ok(())
}

/// Emula [`FRAMES`] frames con `step` y devuelve los frames por segundo alcanzados.
fn measure(
    mut step: impl FnMut() -> Result<(), Box<dyn std::error::Error>>,
) -> Result<f64, Box<dyn std::error::Error>> {
    let start = Instant::now();
    for _ in 0..FRAMES {
        step()?;
    }
    Ok(f64::from(FRAMES) / start.elapsed().as_secs_f64())
}

fn report(label: &str, frames_per_second: f64, real_time: f64) {
    println!(
        "{label:<36} {frames_per_second:>6.0} fps  (×{:.1} la velocidad real)",
        frames_per_second / real_time
    );
}
