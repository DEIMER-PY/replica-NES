//! Tests headless del emulador con la ROM de demostración.

use nes_core::{Button, Emulator, Error, FRAME_BYTES, Player, Region, ResetKind, demo_rom};

/// Frames suficientes para que la demo termine de arrancar y pinte.
const WARMUP_FRAMES: usize = 10;

fn booted() -> Emulator {
    let mut nes = Emulator::new();
    nes.load_rom("demo", &demo_rom()).expect("la demo carga");
    run(&mut nes, WARMUP_FRAMES);
    nes
}

fn run(nes: &mut Emulator, frames: usize) {
    for _ in 0..frames {
        nes.run_frame().expect("el frame se emula");
    }
}

/// Color del frame si es uniforme.
fn solid_color(nes: &mut Emulator) -> Option<[u8; 4]> {
    let frame = nes.frame();
    let (pixels, _) = frame.as_chunks::<4>();
    let first = pixels[0];
    pixels.iter().all(|px| *px == first).then_some(first)
}

fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0, |max, s| max.max(s.abs()))
}

#[test]
fn rejects_invalid_rom() {
    let mut nes = Emulator::new();
    let err = nes.load_rom("basura", &[0u8; 64]).unwrap_err();
    assert!(matches!(err, Error::LoadRom(_)));
    assert!(nes.rom().is_none());
}

#[test]
fn needs_a_rom_to_run() {
    let mut nes = Emulator::new();
    assert!(matches!(nes.run_frame(), Err(Error::NoRom)));
    assert!(matches!(nes.save_state(), Err(Error::NoRom)));
}

#[test]
fn loads_demo_rom() {
    let mut nes = Emulator::new();
    let info = nes.load_rom("demo", &demo_rom()).unwrap();
    assert_eq!(info.name, "demo");
    assert!(!info.battery_backed);
    assert_eq!(info.region, Region::Ntsc);
    assert!(nes.battery_ram().is_none());
}

#[test]
fn renders_a_solid_opaque_frame() {
    let mut nes = booted();
    assert_eq!(nes.frame().len(), FRAME_BYTES);
    let color = solid_color(&mut nes).expect("la demo pinta un color uniforme");
    assert_eq!(color[3], 255, "el frame es opaco");
    assert!(
        color[2] > color[0],
        "sin pulsar A el fondo es azul: {color:?}"
    );
}

#[test]
fn button_changes_picture() {
    let mut nes = booted();
    let idle = solid_color(&mut nes).unwrap();

    nes.set_button(Player::One, Button::A, true);
    run(&mut nes, 3);
    let pressed = solid_color(&mut nes).unwrap();
    assert_ne!(idle, pressed);
    assert!(
        pressed[0] > pressed[2],
        "con A pulsado el fondo es rojo: {pressed:?}"
    );

    nes.set_button(Player::One, Button::A, false);
    run(&mut nes, 3);
    assert_eq!(solid_color(&mut nes).unwrap(), idle);
}

#[test]
fn other_inputs_do_not_trigger() {
    let mut nes = booted();
    let idle = solid_color(&mut nes).unwrap();

    nes.set_button(Player::One, Button::B, true);
    nes.set_button(Player::Two, Button::A, true);
    run(&mut nes, 3);
    assert_eq!(solid_color(&mut nes).unwrap(), idle);
}

#[test]
fn button_produces_sound() {
    let mut nes = booted();
    run(&mut nes, 30);
    let samples_per_frame = nes.audio_samples().len();
    // 48 000 Hz / 60 fps = 800 muestras por frame.
    assert!(
        (700..=900).contains(&samples_per_frame),
        "muestras por frame: {samples_per_frame}"
    );
    let silence = peak(nes.audio_samples());

    nes.set_button(Player::One, Button::A, true);
    run(&mut nes, 10);
    let tone = peak(nes.audio_samples());

    assert!(tone > 0.01, "el tono se oye: {tone}");
    assert!(
        tone > silence * 4.0,
        "tono {tone} frente a silencio {silence}"
    );
}

#[test]
fn save_state_round_trip() {
    let mut nes = booted();
    let idle = solid_color(&mut nes).unwrap();
    let state = nes.save_state().unwrap();
    assert!(!state.is_empty());

    nes.set_button(Player::One, Button::A, true);
    run(&mut nes, 3);
    assert_ne!(solid_color(&mut nes).unwrap(), idle);

    nes.set_button(Player::One, Button::A, false);
    nes.load_state(&state).unwrap();
    run(&mut nes, 1);
    assert_eq!(solid_color(&mut nes).unwrap(), idle);

    assert!(matches!(
        nes.load_state(b"no es un estado"),
        Err(Error::SaveState(_))
    ));
}

#[test]
fn reset_keeps_running() {
    for kind in [ResetKind::Soft, ResetKind::Hard] {
        let mut nes = booted();
        let idle = solid_color(&mut nes).unwrap();
        nes.reset(kind);
        run(&mut nes, WARMUP_FRAMES);
        assert_eq!(solid_color(&mut nes).unwrap(), idle);
    }
}

#[test]
fn unload_rom_stops_emulation() {
    let mut nes = booted();
    nes.unload_rom();
    assert!(nes.rom().is_none());
    assert!(matches!(nes.run_frame(), Err(Error::NoRom)));
}
