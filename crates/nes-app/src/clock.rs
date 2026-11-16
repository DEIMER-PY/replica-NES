//! Ritmo de la emulación: cuántos frames de la NES tocan en cada repintado.
//!
//! La pantalla y la consola no van a la misma frecuencia (un monitor puede ir a 60,
//! 75 o 144 Hz y la NES a 60,1 o 50 fps), así que en cada repintado se acumula el
//! tiempo transcurrido y se emulan los frames enteros que quepan en él.

/// Máximo de frames que se emulan en un solo repintado.
///
/// Si la app se queda atrás más que esto (ventana minimizada, un tirón del sistema),
/// el retraso se descarta en lugar de emularlo todo de golpe.
const MAX_FRAMES_PER_UPDATE: u32 = 3;

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
    /// `now` es el reloj de la app en segundos y `frame_rate` los fps de la consola.
    pub fn frames_due(&mut self, now: f64, frame_rate: f64) -> u32 {
        let elapsed = self.last.map_or(0.0, |last| (now - last).max(0.0));
        self.last = Some(now);
        self.pending += elapsed * frame_rate;

        if self.pending > f64::from(MAX_FRAMES_PER_UPDATE) {
            self.pending = 0.0;
            return MAX_FRAMES_PER_UPDATE;
        }

        let due = self.pending.floor();
        self.pending -= due;
        // `due` está entre 0 y MAX_FRAMES_PER_UPDATE.
        due as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NTSC: f64 = 60.0988;

    #[test]
    fn first_call_runs_nothing() {
        let mut clock = FrameClock::default();
        assert_eq!(clock.frames_due(10.0, NTSC), 0);
    }

    #[test]
    fn matches_console_rate_over_a_second() {
        // Monitor a 144 Hz: la suma de frames en 10 s debe ser la de la consola.
        let mut clock = FrameClock::default();
        let total: u32 = (0..=1440)
            .map(|i| clock.frames_due(f64::from(i) / 144.0, NTSC))
            .sum();
        assert!((600..=601).contains(&total), "frames en 10 s: {total}");
    }

    #[test]
    fn slow_display_runs_several_frames() {
        // Monitor a 30 Hz: dos frames de NES por repintado.
        let mut clock = FrameClock::default();
        clock.frames_due(0.0, 60.0);
        assert_eq!(clock.frames_due(1.0 / 30.0 + 1e-9, 60.0), 2);
    }

    #[test]
    fn long_stall_is_dropped() {
        let mut clock = FrameClock::default();
        clock.frames_due(0.0, NTSC);
        assert_eq!(clock.frames_due(5.0, NTSC), MAX_FRAMES_PER_UPDATE);
        // El retraso no se arrastra al siguiente repintado.
        assert_eq!(clock.frames_due(5.001, NTSC), 0);
    }
}
