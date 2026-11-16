//! Ritmo de la emulación: cuántos frames de la NES tocan en cada repintado.
//!
//! La pantalla y la consola no van a la misma frecuencia (un monitor puede ir a 60,
//! 75 o 144 Hz y la NES a 60,1 o 50 fps), así que en cada repintado se acumula el
//! tiempo transcurrido y se emulan los frames enteros que quepan en él.

/// Máximo de frames que se emulan en un solo repintado a velocidad normal.
///
/// Si la app se queda atrás más que esto (ventana minimizada, un tirón del sistema),
/// el retraso se descarta en lugar de emularlo todo de golpe.
pub const MAX_FRAMES_PER_UPDATE: u32 = 3;

/// Acumulador de tiempo de la emulación.
#[derive(Debug, Default)]
pub struct FrameClock {
    /// Instante del repintado anterior, en segundos.
    last: Option<f64>,
    /// Frames de la NES pendientes de emular (con fracción).
    pending: f64,
}

impl FrameClock {
    /// Devuelve cuántos frames hay que emular ahora.
    ///
    /// `now` es el reloj de la app en segundos, `frame_rate` los fps a los que debe ir
    /// la consola y `max_frames` el tope de frames por repintado.
    pub fn frames_due(&mut self, now: f64, frame_rate: f64, max_frames: u32) -> u32 {
        let elapsed = self.last.map_or(0.0, |last| (now - last).max(0.0));
        self.last = Some(now);
        self.pending += elapsed * frame_rate;

        if self.pending > f64::from(max_frames) {
            self.pending = 0.0;
            return max_frames;
        }

        let due = self.pending.floor();
        self.pending -= due;
        // `due` está entre 0 y `max_frames`.
        due as u32
    }

    /// Olvida el tiempo acumulado, para que al salir de una pausa no se emule de golpe
    /// todo el tiempo que duró.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NTSC: f64 = 60.0988;

    #[test]
    fn first_call_runs_nothing() {
        let mut clock = FrameClock::default();
        assert_eq!(clock.frames_due(10.0, NTSC, MAX_FRAMES_PER_UPDATE), 0);
    }

    #[test]
    fn matches_console_rate_over_a_second() {
        // Monitor a 144 Hz: la suma de frames en 10 s debe ser la de la consola.
        let mut clock = FrameClock::default();
        let total: u32 = (0..=1440)
            .map(|i| clock.frames_due(f64::from(i) / 144.0, NTSC, MAX_FRAMES_PER_UPDATE))
            .sum();
        assert!((600..=601).contains(&total), "frames en 10 s: {total}");
    }

    #[test]
    fn slow_display_runs_several_frames() {
        // Monitor a 30 Hz: dos frames de NES por repintado.
        let mut clock = FrameClock::default();
        clock.frames_due(0.0, 60.0, MAX_FRAMES_PER_UPDATE);
        assert_eq!(
            clock.frames_due(1.0 / 30.0 + 1e-9, 60.0, MAX_FRAMES_PER_UPDATE),
            2
        );
    }

    #[test]
    fn long_stall_is_dropped() {
        let mut clock = FrameClock::default();
        clock.frames_due(0.0, NTSC, MAX_FRAMES_PER_UPDATE);
        assert_eq!(
            clock.frames_due(5.0, NTSC, MAX_FRAMES_PER_UPDATE),
            MAX_FRAMES_PER_UPDATE
        );
        // El retraso no se arrastra al siguiente repintado.
        assert_eq!(clock.frames_due(5.001, NTSC, MAX_FRAMES_PER_UPDATE), 0);
    }

    #[test]
    fn fast_forward_runs_more_frames_per_update() {
        let mut clock = FrameClock::default();
        clock.frames_due(0.0, 240.0, 8);
        assert_eq!(clock.frames_due(1.0 / 60.0 + 1e-9, 240.0, 8), 4);
    }

    #[test]
    fn reset_forgets_elapsed_time() {
        let mut clock = FrameClock::default();
        clock.frames_due(0.0, NTSC, MAX_FRAMES_PER_UPDATE);
        clock.reset();
        assert_eq!(clock.frames_due(100.0, NTSC, MAX_FRAMES_PER_UPDATE), 0);
    }
}
