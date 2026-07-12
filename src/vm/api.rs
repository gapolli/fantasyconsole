use crossbeam_channel::Sender;
use mlua::{FromLua, Lua, Result};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

pub enum AudioCommand {
    PlaySfx {
        channel: usize,
        waveform: u8,
        note: f32,
        duration_ms: u32,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BackendState {
    pub screen_buffer: Vec<u8>,
    pub sprite_sheet: Vec<u8>,
    pub map_buffer: Vec<u8>,
    pub palette_map: [u8; 16],
    pub current_color: u8,
    pub buttons: [bool; 6],
    pub camera_x: i32,
    pub camera_y: i32,
    pub clip_x0: i32,
    pub clip_y0: i32,
    pub clip_x1: i32,
    pub clip_y1: i32,
    pub debug_mode: bool,
    #[serde(skip)]
    pub audio_tx: Option<Sender<AudioCommand>>,
}

impl BackendState {
    pub fn new() -> Self {
        let mut initial_palette = [0u8; 16];
        for i in 0..16 {
            initial_palette[i] = i as u8;
        }

        Self {
            screen_buffer: vec![0u8; 128 * 128],
            sprite_sheet: vec![0u8; 128 * 128],
            map_buffer: vec![0u8; 128 * 64],
            palette_map: initial_palette,
            current_color: 6,
            buttons: [false; 6],
            camera_x: 0,
            camera_y: 0,
            clip_x0: 0,
            clip_y0: 0,
            clip_x1: 127,
            clip_y1: 127,
            debug_mode: false,
            audio_tx: None,
        }
    }

    pub fn mget(&self, cel_x: i32, cel_y: i32) -> u8 {
        if cel_x >= 0 && cel_x < 128 && cel_y >= 0 && cel_y < 64 {
            return self.map_buffer[(cel_y * 128 + cel_x) as usize];
        }
        0
    }

    pub fn mset(&mut self, cel_x: i32, cel_y: i32, tile_id: u8) {
        if cel_x >= 0 && cel_x < 128 && cel_y >= 0 && cel_y < 64 {
            self.map_buffer[(cel_y * 128 + cel_x) as usize] = tile_id;
        }
    }

    #[inline]
    pub fn pset(&mut self, x: i32, y: i32, color: u8) {
        let cam_x = x - self.camera_x;
        let cam_y = y - self.camera_y;
        if cam_x >= self.clip_x0
            && cam_x <= self.clip_x1
            && cam_y >= self.clip_y0
            && cam_y <= self.clip_y1
        {
            if cam_x >= 0 && cam_x < 128 && cam_y >= 0 && cam_y < 128 {
                let mapped_color = self.palette_map[(color & 0x0F) as usize];
                self.screen_buffer[(cam_y * 128 + cam_x) as usize] = mapped_color & 0x0F;
            }
        }
    }

    pub fn palette(&mut self, c0: u8, c1: u8) {
        if c0 < 16 && c1 < 16 {
            self.palette_map[c0 as usize] = c1;
        }
    }

    pub fn pal_reset(&mut self) {
        for i in 0..16 {
            self.palette_map[i] = i as u8;
        }
    }

    pub fn spr_ext(&mut self, n: u32, sx: i32, sy: i32, flip_x: bool, flip_y: bool) {
        let s_x = (n % 16) * 8;
        let s_y = (n / 16) * 8;
        for y in 0..8 {
            for x in 0..8 {
                // Se flip_x for true, lê o sprite de trás para frente horizontalmente
                let read_x = if flip_x { s_x + (7 - x) } else { s_x + x };
                // Se flip_y for true, lê o sprite de ponta-cabeça verticalmente
                let read_y = if flip_y { s_y + (7 - y) } else { s_y + y };

                if read_x < 128 && read_y < 128 {
                    let color = self.sprite_sheet[(read_y * 128 + read_x) as usize];
                    if color != 0 {
                        self.pset(sx + x as i32, sy + y as i32, color);
                    }
                }
            }
        }
    }

    pub fn sspr_ext(
        &mut self,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        dx: i32,
        dy: i32,
        dw: i32,
        dh: i32,
        flip_x: bool,
        flip_y: bool,
    ) {
        if sw <= 0 || sh <= 0 || dw <= 0 || dh <= 0 {
            return;
        }

        for y in 0..dh {
            for x in 0..dw {
                let sample_x = (x * sw) / dw;
                let sample_y = (y * sh) / dh;

                let src_x = if flip_x {
                    sx + (sw - 1 - sample_x)
                } else {
                    sx + sample_x
                };
                let src_y = if flip_y {
                    sy + (sh - 1 - sample_y)
                } else {
                    sy + sample_y
                };

                if src_x >= 0 && src_x < 128 && src_y >= 0 && src_y < 128 {
                    let color = self.sprite_sheet[(src_y * 128 + src_x) as usize];
                    if color != 0 {
                        self.pset(dx + x, dy + y, color);
                    }
                }
            }
        }
    }

    pub fn cls(&mut self, color: u8) {
        let c = color & 0x0F;
        for y in self.clip_y0..=self.clip_y1 {
            for x in self.clip_x0..=self.clip_x1 {
                if x >= 0 && x < 128 && y >= 0 && y < 128 {
                    self.screen_buffer[(y * 128 + x) as usize] = c;
                }
            }
        }
    }

    pub fn camera(&mut self, x: i32, y: i32) {
        self.camera_x = x;
        self.camera_y = y;
    }

    pub fn clip(&mut self, x: i32, y: i32, w: i32, h: i32) {
        if w <= 0 || h <= 0 {
            self.clip_x0 = 0;
            self.clip_y0 = 0;
            self.clip_x1 = 127;
            self.clip_y1 = 127;
        } else {
            self.clip_x0 = x.clamp(0, 127);
            self.clip_y0 = y.clamp(0, 127);
            self.clip_x1 = (x + w - 1).clamp(0, 127);
            self.clip_y1 = (y + h - 1).clamp(0, 127);
        }
    }

    pub fn sspr(&mut self, sx: i32, sy: i32, sw: i32, sh: i32, dx: i32, dy: i32, dw: i32, dh: i32) {
        if sw <= 0 || sh <= 0 || dw <= 0 || dh <= 0 {
            return;
        }

        for y in 0..dh {
            for x in 0..dw {
                let src_x = sx + (x * sw) / dw;
                let src_y = sy + (y * sh) / dh;

                if src_x >= 0 && src_x < 128 && src_y >= 0 && src_y < 128 {
                    let color = self.sprite_sheet[(src_y * 128 + src_x) as usize];
                    // Mantém a cor 0 como transparente padrão
                    if color != 0 {
                        self.pset(dx + x, dy + y, color);
                    }
                }
            }
        }
    }
    pub fn get_ascii_glyph(&self, c: char) -> [u8; 5] {
        match c {
            ' ' => [0b000, 0b000, 0b000, 0b000, 0b000],
            '!' => [0b010, 0b010, 0b010, 0b000, 0b010],
            '"' => [0b101, 0b101, 0b000, 0b000, 0b000],
            '#' => [0b101, 0b111, 0b101, 0b111, 0b101],
            '$' => [0b010, 0b111, 0b100, 0b111, 0b010],
            '%' => [0b101, 0b001, 0b010, 0b100, 0b101],
            '&' => [0b010, 0b101, 0b010, 0b101, 0b111],
            '\'' => [0b010, 0b010, 0b000, 0b000, 0b000],
            '(' => [0b001, 0b010, 0b010, 0b010, 0b001],
            ')' => [0b100, 0b010, 0b010, 0b010, 0b100],
            '*' => [0b010, 0b101, 0b010, 0b101, 0b010],
            '+' => [0b000, 0b010, 0b111, 0b010, 0b000],
            ',' => [0b000, 0b000, 0b000, 0b010, 0b100],
            '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
            '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
            '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
            '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
            '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
            '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
            '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
            '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
            '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
            '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
            '7' => [0b111, 0b001, 0b010, 0b100, 0b100],
            '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
            '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
            ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
            ';' => [0b000, 0b010, 0b000, 0b010, 0b100],
            '<' => [0b001, 0b010, 0b100, 0b010, 0b001],
            '=' => [0b000, 0b111, 0b000, 0b111, 0b000],
            '>' => [0b100, 0b010, 0b001, 0b010, 0b100],
            '?' => [0b111, 0b001, 0b010, 0b000, 0b010],
            '@' => [0b111, 0b101, 0b111, 0b100, 0b111],
            'A' => [0b111, 0b101, 0b111, 0b101, 0b101],
            'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
            'C' => [0b111, 0b100, 0b100, 0b100, 0b111],
            'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
            'E' => [0b111, 0b100, 0b111, 0b100, 0b111],
            'F' => [0b111, 0b100, 0b111, 0b100, 0b100],
            'G' => [0b111, 0b100, 0b101, 0b101, 0b111],
            'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
            'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
            'J' => [0b001, 0b001, 0b001, 0b101, 0b111],
            'K' => [0b101, 0b110, 0b100, 0b110, 0b101],
            'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
            'M' => [0b101, 0b111, 0b101, 0b101, 0b101],
            'N' => [0b111, 0b101, 0b101, 0b101, 0b101],
            'O' => [0b111, 0b101, 0b101, 0b101, 0b111],
            'P' => [0b111, 0b101, 0b111, 0b100, 0b100],
            'Q' => [0b111, 0b101, 0b101, 0b111, 0b001],
            'R' => [0b111, 0b101, 0b110, 0b101, 0b101],
            'S' => [0b111, 0b100, 0b111, 0b001, 0b111],
            'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
            'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
            'V' => [0b101, 0b101, 0b101, 0b010, 0b010],
            'W' => [0b101, 0b101, 0b101, 0b111, 0b101],
            'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
            'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
            'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
            '[' => [0b111, 0b100, 0b100, 0b100, 0b111],
            '\\' => [0b100, 0b100, 0b010, 0b001, 0b001],
            ']' => [0b111, 0b001, 0b001, 0b001, 0b111],
            '^' => [0b010, 0b101, 0b000, 0b000, 0b000],
            '_' => [0b000, 0b000, 0b000, 0b000, 0b111],
            '`' => [0b100, 0b010, 0b000, 0b000, 0b000],
            'a' => [0b000, 0b111, 0b001, 0b111, 0b111],
            'b' => [0b100, 0b110, 0b101, 0b101, 0b110],
            'c' => [0b000, 0b111, 0b100, 0b100, 0b111],
            'd' => [0b001, 0b011, 0b101, 0b101, 0b011],
            'e' => [0b000, 0b111, 0b111, 0b100, 0b111],
            'f' => [0b011, 0b010, 0b110, 0b010, 0b010],
            'g' => [0b000, 0b111, 0b101, 0b111, 0b001],
            'h' => [0b100, 0b110, 0b101, 0b101, 0b101],
            'i' => [0b010, 0b000, 0b010, 0b010, 0b010],
            'j' => [0b001, 0b000, 0b001, 0b001, 0b110],
            'k' => [0b100, 0b101, 0b110, 0b101, 0b101],
            'l' => [0b110, 0b010, 0b010, 0b010, 0b111],
            'm' => [0b000, 0b111, 0b111, 0b101, 0b101],
            'n' => [0b000, 0b110, 0b101, 0b101, 0b101],
            'o' => [0b000, 0b111, 0b101, 0b101, 0b111],
            'p' => [0b000, 0b110, 0b101, 0b110, 0b100],
            'q' => [0b000, 0b011, 0b101, 0b011, 0b001],
            'r' => [0b000, 0b110, 0b101, 0b100, 0b100],
            's' => [0b000, 0b011, 0b100, 0b001, 0b110],
            't' => [0b010, 0b111, 0b010, 0b010, 0b001],
            'u' => [0b000, 0b101, 0b101, 0b101, 0b111],
            'v' => [0b000, 0b101, 0b101, 0b010, 0b010],
            'w' => [0b000, 0b101, 0b101, 0b111, 0b101],
            'x' => [0b000, 0b101, 0b010, 0b101, 0b101],
            'y' => [0b000, 0b101, 0b101, 0b111, 0b001],
            'z' => [0b000, 0b111, 0b010, 0b100, 0b111],
            '{' => [0b001, 0b010, 0b110, 0b010, 0b001],
            '|' => [0b010, 0b010, 0b010, 0b010, 0b010],
            '}' => [0b100, 0b010, 0b011, 0b010, 0b100],
            '~' => [0b010, 0b101, 0b000, 0b000, 0b000],
            _ => [0b111, 0b111, 0b111, 0b111, 0b111],
        }
    }

    pub fn draw_text(&mut self, text: &str, mut start_x: i32, start_y: i32, color: u8) {
        for c in text.chars() {
            let glyph = self.get_ascii_glyph(c);
            for row in 0..5 {
                let bits = glyph[row];
                for col in 0..3 {
                    if (bits & (1 << (2 - col))) != 0 {
                        self.pset(start_x + col, start_y + row as i32, color);
                    }
                }
            }
            start_x += 4;
        }
    }
    pub fn rspr(&mut self, n: u32, dx: i32, dy: i32, angle_rad: f32, scale_x: f32, scale_y: f32) {
        let s_x = ((n % 16) * 8) as f32;
        let s_y = ((n / 16) * 8) as f32;

        let half_w = (4.0 * scale_x).abs();
        let half_h = (4.0 * scale_y).abs();

        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        let bound = (half_w.max(half_h) * 1.4142) as i32 + 1;

        for y in -bound..=bound {
            for x in -bound..=bound {
                let x_scaled = x as f32 / scale_x;
                let y_scaled = y as i32 as f32 / scale_y;

                // Aplica a matriz de rotação inversa (Trigonométrica pura)
                let rot_x = x_scaled * cos_a + y_scaled * sin_a;
                let rot_y = -x_scaled * sin_a + y_scaled * cos_a;

                let src_x = (rot_x + 4.0).floor() as i32;
                let src_y = (rot_y + 4.0).floor() as i32;

                if src_x >= 0 && src_x < 8 && src_y >= 0 && src_y < 8 {
                    let pixel_sheet_x = (s_x + src_x as f32) as i32;
                    let pixel_sheet_y = (s_y + src_y as f32) as i32;

                    if pixel_sheet_x >= 0
                        && pixel_sheet_x < 128
                        && pixel_sheet_y >= 0
                        && pixel_sheet_y < 128
                    {
                        let color =
                            self.sprite_sheet[(pixel_sheet_y * 128 + pixel_sheet_x) as usize];
                        if color != 0 {
                            self.pset(dx + x, dy + y, color);
                        }
                    }
                }
            }
        }
    }
}

impl BackendState {
    // Desenha uma região do mapa de tiles diretamente na tela
    pub fn map(&mut self, cel_x: i32, cel_y: i32, scr_x: i32, scr_y: i32, cel_w: i32, cel_h: i32) {
        for cy in 0..cel_h {
            for cx in 0..cel_w {
                let current_cel_x = cel_x + cx;
                let current_cel_y = cel_y + cy;
                let tile_id = self.mget(current_cel_x, current_cel_y);

                // PICO-8 pula a renderização do tile 0 por padrão (transparente)
                if tile_id != 0 {
                    let draw_x = scr_x + (cx * 8);
                    let draw_y = scr_y + (cy * 8);
                    self.spr(tile_id as u32, draw_x, draw_y);
                }
            }
        }
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u8) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        loop {
            self.pset(x, y, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn circ(&mut self, xc: i32, yc: i32, r: i32, color: u8) {
        if r < 0 {
            return;
        }
        let mut x = 0;
        let mut y = r;
        let mut d = 3 - 2 * r;
        self.draw_circle_pts(xc, yc, x, y, color);
        while y >= x {
            x += 1;
            if d > 0 {
                y -= 1;
                d += 4 * (x - y) + 10;
            } else {
                d += 4 * x + 6;
            }
            self.draw_circle_pts(xc, yc, x, y, color);
        }
    }

    pub fn circfill(&mut self, xc: i32, yc: i32, r: i32, color: u8) {
        if r < 0 {
            return;
        }
        let mut x = 0;
        let mut y = r;
        let mut d = 3 - 2 * r;
        self.draw_circle_lines(xc, yc, x, y, color);
        while y >= x {
            x += 1;
            if d > 0 {
                y -= 1;
                d += 4 * (x - y) + 10;
            } else {
                d += 4 * x + 6;
            }
            self.draw_circle_lines(xc, yc, x, y, color);
        }
    }

    pub fn spr(&mut self, n: u32, sx: i32, sy: i32) {
        let s_x = (n % 16) * 8;
        let s_y = (n / 16) * 8;
        for y in 0..8 {
            for x in 0..8 {
                let pixel_x = s_x + x;
                let pixel_y = s_y + y;
                if pixel_x < 128 && pixel_y < 128 {
                    let color = self.sprite_sheet[(pixel_y * 128 + pixel_x) as usize];
                    if color != 0 {
                        self.pset(sx + x as i32, sy + y as i32, color);
                    }
                }
            }
        }
    }

    pub fn save_state(&self, slot: u32) {
        if let Ok(json) = serde_json::to_string(self) {
            let filename = format!("save_slot_{}.json", slot);
            if let Ok(mut file) = File::create(filename) {
                let _ = file.write_all(json.as_bytes());
            }
        }
    }

    pub fn load_state(&mut self, slot: u32) {
        let filename = format!("save_slot_{}.json", slot);
        if let Ok(file) = File::open(filename) {
            if let Ok(loaded) = serde_json::from_reader::<_, BackendState>(file) {
                self.screen_buffer = loaded.screen_buffer;
                self.sprite_sheet = loaded.sprite_sheet;
                self.map_buffer = loaded.map_buffer;
                self.current_color = loaded.current_color;
                self.camera_x = loaded.camera_x;
                self.camera_y = loaded.camera_y;
                self.clip_x0 = loaded.clip_x0;
                self.clip_y0 = loaded.clip_y0;
                self.clip_x1 = loaded.clip_x1;
                self.clip_y1 = loaded.clip_y1;
            }
        }
    }

    fn draw_circle_pts(&mut self, xc: i32, yc: i32, x: i32, y: i32, c: u8) {
        self.pset(xc + x, yc + y, c);
        self.pset(xc - x, yc + y, c);
        self.pset(xc + x, yc - y, c);
        self.pset(xc - x, yc - y, c);
        self.pset(xc + y, yc + x, c);
        self.pset(xc - y, yc + x, c);
        self.pset(xc + y, yc - x, c);
        self.pset(xc - y, yc - x, c);
    }

    fn draw_circle_lines(&mut self, xc: i32, yc: i32, x: i32, y: i32, c: u8) {
        self.line(xc - x, yc + y, xc + x, yc + y, c);
        self.line(xc - x, yc - y, xc + x, yc - y, c);
        self.line(xc - y, yc + x, xc + y, yc + x, c);
        self.line(xc - y, yc - x, xc + y, yc - x, c);
    }
}

pub fn inject_pico8_api(lua: &Lua, state: Rc<RefCell<BackendState>>) -> Result<()> {
    let globals = lua.globals();

    let s = Rc::clone(&state);
    globals.set(
        "cls",
        lua.create_function(move |_, color: Option<u8>| {
            s.borrow_mut().cls(color.unwrap_or(0));
            Ok(())
        })?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "pset",
        lua.create_function(move |_, (x, y, color): (i32, i32, Option<u8>)| {
            let mut state_mut = s.borrow_mut();
            let c = color.unwrap_or(state_mut.current_color);
            state_mut.pset(x, y, c);
            Ok(())
        })?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "camera",
        lua.create_function(move |_, (x, y): (Option<i32>, Option<i32>)| {
            s.borrow_mut().camera(x.unwrap_or(0), y.unwrap_or(0));
            Ok(())
        })?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "clip",
        lua.create_function(
            move |_, (x, y, w, h): (Option<i32>, Option<i32>, Option<i32>, Option<i32>)| {
                s.borrow_mut().clip(
                    x.unwrap_or(0),
                    y.unwrap_or(0),
                    w.unwrap_or(0),
                    h.unwrap_or(0),
                );
                Ok(())
            },
        )?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "line",
        lua.create_function(
            move |_, (x0, y0, x1, y1, color): (i32, i32, i32, i32, Option<u8>)| {
                let mut state_mut = s.borrow_mut();
                let c = color.unwrap_or(state_mut.current_color);
                state_mut.line(x0, y0, x1, y1, c);
                Ok(())
            },
        )?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "circ",
        lua.create_function(move |_, (x, y, r, color): (i32, i32, i32, Option<u8>)| {
            let mut state_mut = s.borrow_mut();
            let c = color.unwrap_or(state_mut.current_color);
            state_mut.circ(x, y, r, c);
            Ok(())
        })?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "circfill",
        lua.create_function(move |_, (x, y, r, color): (i32, i32, i32, Option<u8>)| {
            let mut state_mut = s.borrow_mut();
            let c = color.unwrap_or(state_mut.current_color);
            state_mut.circfill(x, y, r, c);
            Ok(())
        })?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "pal",
        lua.create_function(move |lua_ctx, args: mlua::MultiValue| {
            let mut s_mut = s.borrow_mut();

            if args.len() >= 2 {
                let mut iter = args.into_iter();
                let val0 = iter.next().unwrap();
                let val1 = iter.next().unwrap();

                if let (Ok(c0), Ok(c1)) = (u8::from_lua(val0, lua_ctx), u8::from_lua(val1, lua_ctx))
                {
                    s_mut.palette(c0, c1);
                }
            } else {
                s_mut.pal_reset();
            }
            Ok(())
        })?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "spr",
        lua.create_function(
            move |_, (n, x, y, flip_x, flip_y): (u32, i32, i32, Option<bool>, Option<bool>)| {
                s.borrow_mut()
                    .spr_ext(n, x, y, flip_x.unwrap_or(false), flip_y.unwrap_or(false));
                Ok(())
            },
        )?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "btn",
        lua.create_function(move |_, button_idx: usize| {
            if button_idx < 6 {
                Ok(s.borrow().buttons[button_idx])
            } else {
                Ok(false)
            }
        })?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "save_game",
        lua.create_function(move |_, slot: u32| {
            s.borrow().save_state(slot);
            Ok(())
        })?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "load_game",
        lua.create_function(move |_, slot: u32| {
            s.borrow_mut().load_state(slot);
            Ok(())
        })?,
    )?;

    // BINDINGS DA API DE MAPA
    let s = Rc::clone(&state);
    globals.set(
        "mget",
        lua.create_function(move |_, (x, y): (i32, i32)| Ok(s.borrow().mget(x, y)))?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "mset",
        lua.create_function(move |_, (x, y, tile_id): (i32, i32, u8)| {
            s.borrow_mut().mset(x, y, tile_id);
            Ok(())
        })?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "map",
        lua.create_function(
            move |_, (cel_x, cel_y, scr_x, scr_y, cel_w, cel_h): (i32, i32, i32, i32, i32, i32)| {
                s.borrow_mut().map(cel_x, cel_y, scr_x, scr_y, cel_w, cel_h);
                Ok(())
            },
        )?,
    )?;

    // BINDINGS DA API DE ÁUDIO
    let s = Rc::clone(&state);
    globals.set(
        "sfx",
        lua.create_function(move |_, (n, channel_idx): (i32, Option<usize>)| {
            let state_ref = s.borrow();
            if let Some(ref tx) = state_ref.audio_tx {
                let target_channel = channel_idx.unwrap_or(0) & 3;
                let freq = match n {
                    0 => 261.63,
                    1 => 293.66,
                    2 => 329.63,
                    3 => 349.23,
                    4 => 392.00,
                    _ => 440.00,
                };
                let wave_type = (n % 4) as u8;
                let _ = tx.send(AudioCommand::PlaySfx {
                    channel: target_channel,
                    waveform: wave_type,
                    note: freq,
                    duration_ms: 150,
                });
            }
            Ok(())
        })?,
    )?;
    let s = Rc::clone(&state);
    globals.set(
        "sspr",
        lua.create_function(
            move |_,
                  (sx, sy, sw, sh, dx, dy, dw, dh, flip_x, flip_y): (
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
                Option<bool>,
                Option<bool>,
            )| {
                s.borrow_mut().sspr_ext(
                    sx,
                    sy,
                    sw,
                    sh,
                    dx,
                    dy,
                    dw,
                    dh,
                    flip_x.unwrap_or(false),
                    flip_y.unwrap_or(false),
                );
                Ok(())
            },
        )?,
    )?;
    let s = Rc::clone(&state);
    globals.set(
        "print",
        lua.create_function(
            move |_, (text, x, y, color): (String, i32, i32, Option<u8>)| {
                let mut state_mut = s.borrow_mut();
                let c = color.unwrap_or(state_mut.current_color);
                state_mut.draw_text(&text, x, y, c);
                Ok(())
            },
        )?,
    )?;
    let s = Rc::clone(&state);
    globals.set(
        "rspr",
        lua.create_function(
            move |_,
                  (n, x, y, angle, scale_x, scale_y): (
                u32,
                i32,
                i32,
                f32,
                Option<f32>,
                Option<f32>,
            )| {
                s.borrow_mut().rspr(
                    n,
                    x,
                    y,
                    angle,
                    scale_x.unwrap_or(1.0),
                    scale_y.unwrap_or(1.0),
                );
                Ok(())
            },
        )?,
    )?;

    globals.set("music", lua.create_function(|_, _n: i32| Ok(()))?)?;

    Ok(())
}
