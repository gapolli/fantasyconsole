pub enum Waveform {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

pub struct Oscillator {
    pub waveform: Waveform,
    pub frequency: f32,
    pub volume: f32,
    phase: f32,
    sample_rate: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            waveform: Waveform::Square,
            frequency: 440.0,
            volume: 0.1,
            phase: 0.0,
            sample_rate,
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        if self.frequency <= 0.0 { return 0.0; }
        
        let value = match self.waveform {
            Waveform::Sine => (self.phase * 2.0 * std::f32::consts::PI).sin(),
            Waveform::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },
            Waveform::Triangle => 1.0 - 4.0 * (self.phase - 0.5).abs(),
            Waveform::Sawtooth => 2.0 * (self.phase - self.phase.floor()) - 1.0,
        };

        // Avança a fase baseado na frequência atual do som
        self.phase = (self.phase + self.frequency / self.sample_rate) % 1.0;
        value * self.volume
    }
}
