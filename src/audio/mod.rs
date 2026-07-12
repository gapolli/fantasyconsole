pub mod sfx;

use sdl2::audio::AudioCallback;
use sfx::Oscillator;

pub struct SoundChannel {
    pub osc: Oscillator,
    pub active: bool,
    pub remaining_samples: u32,
}

pub struct AudioMixer {
    pub channels: [SoundChannel; 4],
}

impl AudioCallback for AudioMixer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            let mut mixed = 0.0;
            for channel in self.channels.iter_mut() {
                if channel.active {
                    mixed += channel.osc.next_sample();
                    if channel.remaining_samples > 0 {
                        channel.remaining_samples -= 1;
                        if channel.remaining_samples == 0 {
                            channel.active = false;
                        }
                    }
                }
            }
            *sample = mixed.clamp(-1.0, 1.0);
        }
    }
}
