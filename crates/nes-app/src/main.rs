fn main() {
    println!(
        "replica-NES {} — pantalla {}x{}",
        env!("CARGO_PKG_VERSION"),
        nes_core::SCREEN_WIDTH,
        nes_core::SCREEN_HEIGHT
    );
}
