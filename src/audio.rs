//! Short, self-contained PCM feedback cues. Buffers outlive asynchronous playback.
use crate::{game::FeedbackKind, win32::*};
pub struct Audio {
    correct: Vec<u8>,
    wrong: Vec<u8>,
}
impl Audio {
    pub fn new() -> Self {
        Self {
            correct: wave(&[659.25, 783.99, 1046.50], 0.075, 0.13),
            wrong: wave(&[349.23, 261.63], 0.11, 0.11),
        }
    }
    pub fn play(&self, kind: FeedbackKind) -> bool {
        let bytes = match kind {
            FeedbackKind::Correct => &self.correct,
            FeedbackKind::Wrong => &self.wrong,
        };
        unsafe { PlaySoundW(bytes.as_ptr() as *const u16, NULL, 0x0007) != 0 }
    }
    pub fn stop(&self) {
        unsafe {
            PlaySoundW(std::ptr::null(), NULL, 0);
        }
    }
}
impl Drop for Audio {
    fn drop(&mut self) {
        self.stop();
    }
}
fn wave(notes: &[f64], seconds: f64, volume: f64) -> Vec<u8> {
    let rate = 22050u32;
    let count = (seconds * rate as f64) as usize;
    let mut samples = Vec::with_capacity(notes.len() * count * 2);
    for &frequency in notes {
        for i in 0..count {
            let t = i as f64 / rate as f64;
            let envelope = (t / 0.006).min(1.0) * ((seconds - t) / 0.025).clamp(0.0, 1.0);
            let tone = (std::f64::consts::TAU * frequency * t).sin()
                + 0.18 * (std::f64::consts::TAU * frequency * 2.0 * t).sin();
            let value = (tone * envelope * volume * i16::MAX as f64) as i16;
            samples.extend_from_slice(&value.to_le_bytes());
        }
    }
    let mut bytes = Vec::with_capacity(samples.len() + 44);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + samples.len() as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&rate.to_le_bytes());
    bytes.extend_from_slice(&(rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(samples.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&samples);
    bytes
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cues_are_distinct_complete_pcm_files() {
        let a = Audio::new();
        assert_ne!(a.correct, a.wrong);
        for bytes in [&a.correct, &a.wrong] {
            assert_eq!(&bytes[0..4], b"RIFF");
            assert_eq!(&bytes[8..12], b"WAVE");
            assert_eq!(
                u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize,
                bytes.len() - 44
            );
            assert!(bytes[44..]
                .chunks_exact(2)
                .all(|v| i16::from_le_bytes([v[0], v[1]]).abs() < 6000));
        }
    }
}
