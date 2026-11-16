//! Audio: cola de muestras entre el emulador y la tarjeta de sonido.
//!
//! El emulador produce muestras en el hilo de la interfaz, un frame cada ~16 ms, y la
//! tarjeta de sonido las pide desde su propio hilo en bloques pequeños. Entre ambos
//! hay una cola ([`SampleQueue`]) que absorbe esa diferencia de ritmo.

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, PoisonError},
};

use cpal::{
    FromSample, SampleFormat, SizedSample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

/// Audio que se intenta mantener en cola, en segundos. Es la latencia del sonido.
const TARGET_LATENCY: f32 = 0.06;
/// Tope de la cola, en segundos. Lo que llegue por encima se descarta.
const MAX_LATENCY: f32 = 0.15;

/// Cola de muestras mono con capacidad limitada.
#[derive(Debug)]
struct SampleQueue {
    samples: VecDeque<f32>,
    capacity: usize,
    /// Última muestra entregada, para apagar el sonido sin chasquido si la cola se vacía.
    last: f32,
}

impl SampleQueue {
    fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity),
            capacity,
            last: 0.0,
        }
    }

    /// Añade muestras. Si la cola se llenaría, descarta las más antiguas: es preferible
    /// perder un instante de sonido a que el audio se retrase respecto a la imagen.
    fn push(&mut self, samples: &[f32]) {
        self.samples.extend(samples);
        let excess = self.samples.len().saturating_sub(self.capacity);
        self.samples.drain(..excess);
    }

    /// Siguiente muestra. Si no hay, baja suavemente hacia el silencio.
    fn pop(&mut self) -> f32 {
        self.last = self.samples.pop_front().unwrap_or(self.last * 0.995);
        self.last
    }

    fn len(&self) -> usize {
        self.samples.len()
    }

    fn clear(&mut self) {
        self.samples.clear();
    }
}

/// Estado compartido con el hilo de audio.
#[derive(Debug)]
struct Shared {
    queue: SampleQueue,
    /// Volumen de 0 a 1 que se aplica al reproducir.
    gain: f32,
}

/// Salida de audio hacia el dispositivo por defecto del sistema.
pub struct AudioOutput {
    /// El stream se detiene al soltarse, así que vive mientras viva la salida.
    _stream: cpal::Stream,
    shared: Arc<Mutex<Shared>>,
    sample_rate: u32,
}

impl AudioOutput {
    /// Abre el dispositivo de salida por defecto.
    ///
    /// # Errors
    ///
    /// Devuelve un mensaje para el usuario si no hay dispositivo o no se puede abrir.
    /// La app debe seguir funcionando sin sonido en ese caso.
    pub fn open() -> Result<Self, String> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or("No hay ningún dispositivo de salida de audio")?;
        let supported = device
            .default_output_config()
            .map_err(|err| format!("No se pudo consultar el dispositivo de audio: {err}"))?;

        let sample_rate = supported.sample_rate();
        let capacity = (sample_rate as f32 * MAX_LATENCY) as usize;
        let shared = Arc::new(Mutex::new(Shared {
            queue: SampleQueue::new(capacity),
            gain: 1.0,
        }));

        let stream = match supported.sample_format() {
            SampleFormat::F32 => build_stream::<f32>(&device, &supported, &shared),
            SampleFormat::I16 => build_stream::<i16>(&device, &supported, &shared),
            SampleFormat::U16 => build_stream::<u16>(&device, &supported, &shared),
            SampleFormat::I32 => build_stream::<i32>(&device, &supported, &shared),
            other => Err(format!("Formato de audio no soportado: {other}")),
        }?;
        stream
            .play()
            .map_err(|err| format!("No se pudo iniciar el audio: {err}"))?;

        Ok(Self {
            _stream: stream,
            shared,
            sample_rate,
        })
    }

    /// Frecuencia de muestreo del dispositivo, en Hz. El emulador debe generar a esta.
    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Encola las muestras mono de un frame emulado.
    pub fn push(&self, samples: &[f32]) {
        self.lock().queue.push(samples);
    }

    /// Fija el volumen (0 a 1). Con `muted` no suena nada.
    pub fn set_volume(&self, volume: f32, muted: bool) {
        self.lock().gain = if muted { 0.0 } else { volume.clamp(0.0, 1.0) };
    }

    /// Descarta lo encolado, por ejemplo al cambiar de ROM.
    pub fn clear(&self) {
        self.lock().queue.clear();
    }

    /// Cuánto se aparta la cola de su nivel objetivo, de -1 (vacía) a +1 o más (llena).
    ///
    /// La app lo usa para compensar: con la cola baja emula un poco más rápido y con la
    /// cola alta un poco más lento, de forma inaudible.
    pub fn fill_error(&self) -> f32 {
        let target = self.sample_rate as f32 * TARGET_LATENCY;
        (self.lock().queue.len() as f32 - target) / target
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Shared> {
        // Si el hilo de audio entró en pánico, los datos siguen siendo utilizables.
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Crea el stream de salida para el tipo de muestra `T` del dispositivo.
fn build_stream<T>(
    device: &cpal::Device,
    supported: &cpal::SupportedStreamConfig,
    shared: &Arc<Mutex<Shared>>,
) -> Result<cpal::Stream, String>
where
    T: SizedSample + FromSample<f32>,
{
    let channels = usize::from(supported.channels());
    let shared = Arc::clone(shared);
    device
        .build_output_stream(
            supported.config(),
            move |output: &mut [T], _| {
                let mut shared = shared.lock().unwrap_or_else(PoisonError::into_inner);
                let gain = shared.gain;
                // La NES es mono: la misma muestra va a todos los canales.
                for frame in output.chunks_mut(channels) {
                    let sample = T::from_sample(shared.queue.pop() * gain);
                    frame.fill(sample);
                }
            },
            // Un fallo del dispositivo (auriculares desconectados...) deja la app sin
            // sonido, pero no debe cerrarla.
            |_err| {},
            None,
        )
        .map_err(|err| format!("No se pudo abrir la salida de audio: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_is_first_in_first_out() {
        let mut queue = SampleQueue::new(8);
        queue.push(&[0.1, 0.2, 0.3]);
        assert_eq!(queue.len(), 3);
        assert_eq!([queue.pop(), queue.pop(), queue.pop()], [0.1, 0.2, 0.3]);
    }

    #[test]
    fn queue_drops_oldest_when_full() {
        let mut queue = SampleQueue::new(4);
        queue.push(&[1.0, 2.0, 3.0]);
        queue.push(&[4.0, 5.0, 6.0]);
        assert_eq!(queue.len(), 4);
        assert_eq!(queue.pop(), 3.0);
    }

    #[test]
    fn empty_queue_fades_to_silence() {
        let mut queue = SampleQueue::new(4);
        queue.push(&[0.5]);
        assert_eq!(queue.pop(), 0.5);

        let first = queue.pop();
        assert!(first < 0.5 && first > 0.49, "baja poco a poco: {first}");
        let later = (0..4000).map(|_| queue.pop()).last().unwrap();
        assert!(later.abs() < 1e-6, "acaba en silencio: {later}");
    }

    #[test]
    fn clear_empties_the_queue() {
        let mut queue = SampleQueue::new(4);
        queue.push(&[1.0, 2.0]);
        queue.clear();
        assert_eq!(queue.len(), 0);
    }
}
