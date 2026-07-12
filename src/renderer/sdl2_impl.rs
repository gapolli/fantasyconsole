use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;

pub const PALETTE: [Color; 16] = [
    Color::RGB(0x1D, 0x2C, 0x4D), Color::RGB(0xFC, 0xDA, 0xB6),
    Color::RGB(0xEC, 0x5D, 0x57), Color::RGB(0xF6, 0xB7, 0x04),
    Color::RGB(0x76, 0xC4, 0x42), Color::RGB(0x23, 0x85, 0xBE),
    Color::RGB(0xBB, 0x00, 0xCF), Color::RGB(0x00, 0x00, 0x00),
    Color::RGB(0x69, 0xD2, 0x27), Color::RGB(0xA7, 0xDD, 0x00),
    Color::RGB(0xEF, 0x35, 0x2C), Color::RGB(0xFF, 0x89, 0x36),
    Color::RGB(0xFF, 0xC1, 0x07), Color::RGB(0x7E, 0xB0, 0xD6),
    Color::RGB(0x4A, 0x8B, 0xC2), Color::RGB(0xFF, 0xFF, 0xFF),
];

pub struct WindowSystem {
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
}

impl WindowSystem {
    pub fn init(scale: u32) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("FantasyConsole", 128 * scale, 128 * scale)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().present_vsync().build().map_err(|e| e.to_string())?;
        canvas.set_scale(scale as f32, scale as f32).map_err(|e| e.to_string())?;

        let event_pump = sdl_context.event_pump()?;

        Ok(WindowSystem { canvas, event_pump })
    }

    pub fn render(&mut self, buffer: &[u8]) -> Result<(), String> {
        for y in 0..128 {
            for x in 0..128 {
                let color_idx = buffer[y * 128 + x] as usize;
                let color = PALETTE[color_idx & 0x0F];
                self.canvas.set_draw_color(color);
                self.canvas.draw_point((x as i32, y as i32))?;
            }
        }
        self.canvas.present();
        Ok(())
    }
}
