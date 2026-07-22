// src/audio/mod.rs
pub mod sfx;

use sdl2::audio::AudioCallback;
use sfx::Oscillator;

pub struct SoundChannel {
    pub osc: Oscillator,
    pub active: bool,
    pub remaining_samples: u32,
    
    // --- NOVOS CAMPOS PARA O ARPEGGIATOR ---
    pub arpeggio_notes: Vec<f32>,
    pub current_note_idx: usize,
    pub samples_per_tick: u32,
    pub tick_counter: u32,
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
                    
                    // --- ENGENHARIA DO ARPEGGIATOR ---
                    // Se houver mais de uma nota na lista, alternamos a frequência do oscilador
                    if channel.arpeggio_notes.len() > 1 {
                        channel.tick_counter += 1;
                        if channel.tick_counter >= channel.samples_per_tick {
                            channel.tick_counter = 0;
                            // Avança o índice de forma circular pelas notas enviadas
                            channel.current_note_idx = (channel.current_note_idx + 1) % channel.arpeggio_notes.len();
                            // Atualiza o campo de frequência pública do seu Oscillator
                            channel.osc.frequency = channel.arpeggio_notes[channel.current_note_idx];
                        }
                    }
                    // ---------------------------------

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
