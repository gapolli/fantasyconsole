#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleMode {
    Pico8,
    Tic80,
}

pub struct EngineConfig {
    pub mode: ConsoleMode,
    pub screen_width: i32,
    pub screen_height: i32,
    pub palette_size: usize,
}

impl EngineConfig {
    pub fn for_mode(mode: ConsoleMode) -> Self {
        match mode {
            ConsoleMode::Pico8 => Self {
                mode,
                screen_width: 128,
                screen_height: 128,
                palette_size: 16,
            },
            ConsoleMode::Tic80 => Self {
                mode,
                screen_width: 240,
                screen_height: 136,
                palette_size: 16,
            },
        }
    }
}
