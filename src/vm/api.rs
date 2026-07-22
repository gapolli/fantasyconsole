use crossbeam_channel::Sender;
use mlua::{FromLua, Lua, Result};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

// Vinculação com o módulo core polimórfico
use crate::core::{ConsoleMode, EngineConfig};

pub enum AudioCommand {
    PlaySfx {
        channel: usize,
        waveform: u8,
        notes: Vec<f32>,
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
    pub buttons: [[bool; 6]; 4], // Matriz de 4 jogadores x 6 botões
    pub camera_x: i32,
    pub camera_y: i32,
    pub clip_x0: i32,
    pub clip_y0: i32,
    pub clip_x1: i32,
    pub clip_y1: i32,
    pub debug_mode: bool,
    pub target_width: i32,  // Alvo dinâmico de largura (128 ou 240)
    pub target_height: i32, // Alvo dinâmico de altura (128 ou 136)
    #[serde(skip)]
    pub audio_tx: Option<Sender<AudioCommand>>,
}

impl BackendState {
    pub fn new(mode: ConsoleMode) -> Self {
        let config = EngineConfig::for_mode(mode);
        let mut initial_palette = [0u8; 16];
        for i in 0..16 {
            initial_palette[i] = i as u8;
        }

        Self {
            screen_buffer: vec![0u8; (config.screen_width * config.screen_height) as usize],
            sprite_sheet: vec![0u8; 256 * 256],
            map_buffer: vec![0u8; 240 * 136],
            palette_map: initial_palette,
            current_color: 6,
            buttons: [[false; 6]; 4],
            camera_x: 0,
            camera_y: 0,
            clip_x0: 0,
            clip_y0: 0,
            clip_x1: config.screen_width - 1,
            clip_y1: config.screen_height - 1,
            debug_mode: false,
            target_width: config.screen_width,
            target_height: config.screen_height,
            audio_tx: None,
        }
    }

    // Leitura de células de mapas de forma polimórfica
    pub fn mget(&self, cel_x: i32, cel_y: i32) -> u8 {
        if cel_x >= 0 && cel_x < 240 && cel_y >= 0 && cel_y < 136 {
            return self.map_buffer[(cel_y * 240 + cel_x) as usize];
        }
        0
    }

    // Escrita de células de mapas de forma polimórfica
    pub fn mset(&mut self, cel_x: i32, cel_y: i32, tile_id: u8) {
        if cel_x >= 0 && cel_x < 240 && cel_y >= 0 && cel_y < 136 {
            self.map_buffer[(cel_y * 240 + cel_x) as usize] = tile_id;
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
            if cam_x >= 0 && cam_x < self.target_width && cam_y >= 0 && cam_y < self.target_height {
                let idx = (cam_y * self.target_width + cam_x) as usize;
                let mapped_color = self.palette_map[(color & 0x0F) as usize];
                self.screen_buffer[idx] = mapped_color & 0x0F;
            }
        }
    }

    pub fn cls(&mut self, color: u8) {
        let c = color & 0x0F;
        for y in self.clip_y0..=self.clip_y1 {
            for x in self.clip_x0..=self.clip_x1 {
                if x >= 0 && x < self.target_width && y >= 0 && y < self.target_height {
                    let idx = (y * self.target_width + x) as usize;
                    let mapped_color = self.palette_map[c as usize];
                    self.screen_buffer[idx] = mapped_color & 0x0F;
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
            self.clip_x1 = self.target_width - 1;
            self.clip_y1 = self.target_height - 1;
        } else {
            self.clip_x0 = x.clamp(0, self.target_width - 1);
            self.clip_y0 = y.clamp(0, self.target_height - 1);
            self.clip_x1 = (x + w - 1).clamp(0, self.target_width - 1);
            self.clip_y1 = (y + h - 1).clamp(0, self.target_height - 1);
        }
    }
}

impl BackendState {
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

    pub fn map(&mut self, cel_x: i32, cel_y: i32, scr_x: i32, scr_y: i32, cel_w: i32, cel_h: i32) {
        for cy in 0..cel_h {
            for cx in 0..cel_w {
                let current_cel_x = cel_x + cx;
                let current_cel_y = cel_y + cy;
                let tile_id = self.mget(current_cel_x, current_cel_y);

                if tile_id != 0 {
                    let draw_x = scr_x + (cx * 8);
                    let draw_y = scr_y + (cy * 8);
                    self.spr_ext(tile_id as u32, draw_x, draw_y, false, false);
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

    pub fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u8) {
        // Ordena corretamente as coordenadas (min para max)
        let start_x = x0.min(x1);
        let end_x = x0.max(x1);
        let start_y = y0.min(y1);
        let end_y = y0.max(y1);

        // Se o retângulo for apenas um ponto ou uma linha, simplifica usando pset ou line
        if start_x == end_x && start_y == end_y {
            self.pset(start_x, start_y, color);
            return;
        }

        // Desenha as 4 bordas de forma otimizada usando scanlines horizontais e verticais diretas
        // Borda superior e inferior (linhas horizontais)
        self.line(start_x, start_y, end_x, start_y, color);
        self.line(start_x, end_y, end_x, end_y, color);

        // Borda esquerda e direita (linhas verticais - pulando os cantos já desenhados)
        if end_y - start_y > 1 {
            self.line(start_x, start_y + 1, start_x, end_y - 1, color);
            self.line(end_x, start_y + 1, end_x, end_y - 1, color);
        }
    }
    pub fn rectfill(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u8) {
        // Garante que ordenamos corretamente as coordenadas (min para max)
        let start_x = x0.min(x1);
        let end_x = x0.max(x1);
        let start_y = y0.min(y1);
        let end_y = y0.max(y1);

        // Aplica o deslocamento da câmera nas coordenadas
        let cam_start_x = start_x - self.camera_x;
        let cam_end_x = end_x - self.camera_x;
        let cam_start_y = start_y - self.camera_y;
        let cam_end_y = end_y - self.camera_y;

        // Calcula a interseção entre o retângulo e os limites do CLIP e do hardware alvo
        let clip_min_x = cam_start_x.max(self.clip_x0).max(0);
        let clip_max_x = cam_end_x.min(self.clip_x1).min(self.target_width - 1);
        let clip_min_y = cam_start_y.max(self.clip_y0).max(0);
        let clip_max_y = cam_end_y.min(self.clip_y1).min(self.target_height - 1);

        // Se o retângulo estiver completamente fora da tela recortada, aborta cedo
        if clip_min_x > clip_max_x || clip_min_y > clip_max_y {
            return;
        }

        // Resolve o índice de cor mapeado na paleta atual de hardware (0x0F limita a 16 cores)
        let mapped_color = self.palette_map[(color & 0x0F) as usize] & 0x0F;

        // Desenha linha por linha usando scanlines otimizadas
        for y in clip_min_y..=clip_max_y {
            let row_offset = (y * self.target_width) as usize;
            for x in clip_min_x..=clip_max_x {
                let idx = row_offset + x as usize;
                self.screen_buffer[idx] = mapped_color;
            }
        }
    }

    pub fn polygon(&mut self, cx: i32, cy: i32, radius: i32, sides: i32, angle_rad: f32, color: u8) {
        // Um polígono precisa de pelo menos 3 lados para existir
        if sides < 3 || radius <= 0 {
            return;
        }

        let mut prev_x = 0;
        let mut prev_y = 0;
        let mut first_x = 0;
        let mut first_y = 0;

        let sides_f = sides as f32;
        let cx_f = cx as f32;
        let cy_f = cy as f32;
        let r_f = radius as f32;

        for i in 0..=sides {
            // Divide os 360 graus (2 * PI) uniformemente entre o número de lados
            let current_angle = angle_rad + (i as f32 * 2.0 * std::f32::consts::PI) / sides_f;
            
            // Calcula a posição do vértice atual usando coordenadas polares
            let x = (cx_f + current_angle.cos() * r_f).round() as i32;
            let y = (cy_f + current_angle.sin() * r_f).round() as i32;

            if i == 0 {
                // Guarda o primeiro ponto para fechar o polígono no final
                first_x = x;
                first_y = y;
            } else {
                // Desenha a linha ligando o vértice anterior ao atual
                self.line(prev_x, prev_y, x, y, color);
            }

            prev_x = x;
            prev_y = y;
        }

        // Garante o fechamento perfeito do último lado voltando ao início
        self.line(prev_x, prev_y, first_x, first_y, color);
    }

    pub fn polyfill(&mut self, cx: i32, cy: i32, radius: i32, sides: i32, angle_rad: f32, color: u8) {
        if sides < 3 || radius <= 0 {
            return;
        }

        // 1. Calcular e armazenar todos os vértices do polígono regular
        let mut vertices = Vec::with_capacity(sides as usize);
        let sides_f = sides as f32;
        let cx_f = cx as f32;
        let cy_f = cy as f32;
        let r_f = radius as f32;

        let mut min_y = self.target_height;
        let mut max_y = 0;

        for i in 0..sides {
            let current_angle = angle_rad + (i as f32 * 2.0 * std::f32::consts::PI) / sides_f;
            let x = (cx_f + current_angle.cos() * r_f).round() as i32;
            let y = (cy_f + current_angle.sin() * r_f).round() as i32;
            
            vertices.push((x, y));

            // Atualiza os limites verticais (AABB) do polígono para limitar a varredura
            if y < min_y { min_y = y; }
            if y > max_y { max_y = y; }
        }

        // Clipagem vertical rápida contra os limites físicos do hardware e do clip ativo
        let scan_start = min_y.max(self.clip_y0).max(0);
        let scan_end = max_y.min(self.clip_y1).min(self.target_height - 1);

        if scan_start > scan_end {
            return;
        }

        // 2. Loop de varredura por scanlines horizontais
        let mut intersections = Vec::with_capacity(sides as usize);

        for y in scan_start..=scan_end {
            intersections.clear();

            // Percorre todas as arestas do polígono
            for i in 0..sides as usize {
                let p1 = vertices[i];
                let p2 = vertices[(i + 1) % sides as usize]; // Próximo vértice (fecha o loop)

                // Verifica se a scanline horizontal atual cruza a aresta verticalmente
                if (p1.1 <= y && p2.1 > y) || (p2.1 <= y && p1.1 > y) {
                    // Evita divisão por zero se a linha for perfeitamente horizontal
                    if p1.1 != p2.1 {
                        // Interpolação linear para encontrar o valor exato de X na interseção
                        let intersect_x = p1.0 + (y - p1.1) * (p2.0 - p1.0) / (p2.1 - p1.1);
                        intersections.push(intersect_x);
                    }
                }
            }

            // 3. Ordena os pontos de interseção da esquerda para a direita (eixo X)
            intersections.sort_unstable();

            // 4. Desenha as linhas horizontais de preenchimento (pares de dentro do polígono)
            for chunk in intersections.chunks_exact(2) {
                let mut x0 = chunk[0];
                let mut x1 = chunk[1];

                // Clipagem horizontal manual rápida para a scanline atual
                x0 = x0.max(self.clip_x0).max(0);
                x1 = x1.min(self.clip_x1).min(self.target_width - 1);

                if x0 <= x1 {
                    self.line(x0, y, x1, y, color);
                }
            }
        }
    }

    pub fn spr_ext(&mut self, n: u32, sx: i32, sy: i32, flip_x: bool, flip_y: bool) {
        let s_x = (n % 16) * 8;
        let s_y = (n / 16) * 8;
        for y in 0..8 {
            for x in 0..8 {
                let read_x = if flip_x { s_x + (7 - x) } else { s_x + x };
                let read_y = if flip_y { s_y + (7 - y) } else { s_y + y };

                if read_x < 256 && read_y < 256 {
                    let color = self.sprite_sheet[(read_y * 256 + read_x) as usize];
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

                if src_x >= 0 && src_x < 256 && src_y >= 0 && src_y < 256 {
                    let color = self.sprite_sheet[(src_y * 256 + src_x) as usize];
                    if color != 0 {
                        self.pset(dx + x, dy + y, color);
                    }
                }
            }
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
                let y_scaled = y as f32 / scale_y;
                let rot_x = x_scaled * cos_a + y_scaled * sin_a;
                let rot_y = -x_scaled * sin_a + y_scaled * cos_a;
                let src_x = (rot_x + 4.0).floor() as i32;
                let src_y = (rot_y + 4.0).floor() as i32;

                if src_x >= 0 && src_x < 8 && src_y >= 0 && src_y < 8 {
                    let pixel_sheet_x = (s_x + src_x as f32) as i32;
                    let pixel_sheet_y = (s_y + src_y as f32) as i32;

                    if pixel_sheet_x >= 0
                        && pixel_sheet_x < 256
                        && pixel_sheet_y >= 0
                        && pixel_sheet_y < 256
                    {
                        let color =
                            self.sprite_sheet[(pixel_sheet_y * 256 + pixel_sheet_x) as usize];
                        if color != 0 {
                            self.pset(dx + x, dy + y, color);
                        }
                    }
                }
            }
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

impl BackendState {
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
        "rect",
        lua.create_function(
            move |_, (x0, y0, x1, y1, color): (i32, i32, i32, i32, Option<u8>)| {
                let mut state_mut = s.borrow_mut();
                let c = color.unwrap_or(state_mut.current_color);
                state_mut.rect(x0, y0, x1, y1, c);
                Ok(())
            },
        )?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "rectfill",
        lua.create_function(
            move |_, (x0, y0, x1, y1, color): (i32, i32, i32, i32, Option<u8>)| {
                let mut state_mut = s.borrow_mut();
                let c = color.unwrap_or(state_mut.current_color);
                state_mut.rectfill(x0, y0, x1, y1, c);
                Ok(())
            },
        )?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "polygon",
        lua.create_function(
            move |_, (cx, cy, radius, sides, angle, color): (i32, i32, i32, i32, Option<f32>, Option<u8>)| {
                let mut state_mut = s.borrow_mut();
                let c = color.unwrap_or(state_mut.current_color);
                let a = angle.unwrap_or(0.0);
                state_mut.polygon(cx, cy, radius, sides, a, c);
                Ok(())
            },
        )?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "polyfill",
        lua.create_function(
            move |_, (cx, cy, radius, sides, angle, color): (i32, i32, i32, i32, Option<f32>, Option<u8>)| {
                let mut state_mut = s.borrow_mut();
                let c = color.unwrap_or(state_mut.current_color);
                let a = angle.unwrap_or(0.0);
                state_mut.polyfill(cx, cy, radius, sides, a, c);
                Ok(())
            },
        )?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "sfx",
        lua.create_function(
            move |_, (sfx_id, channel_arg, offset, length): (i32, Option<i32>, Option<i32>, Option<i32>)| {
                let state_ref = s.borrow();
                
                // Se o transmissor de áudio crossbeam não estiver configurado, aborta com segurança
                let tx = match &state_ref.audio_tx {
                    Some(t) => t,
                    None => return Ok(()),
                };

                // Padrão inspirado em consoles de fantasia:
                // Se o canal não for especificado (-1 ou None), escolhemos um automaticamente (ex: Canal 0)
                let channel = match channel_arg {
                    Some(c) => if c < 0 { 0 } else { c as usize & 3 }, // Limita entre os 4 canais físicos (0 a 3)
                    None => 0,
                };

                // --- SISTEMA DE NOTAS E SÍNTESE PROVISÓRIA ---
                // No futuro, sfx_id lerá dados de ondas pré-gravadas na RAM do console (Fase 5 - Sound Editor)
                // Por enquanto, faremos o sfx_id atuar diretamente como um gerador de tom/frequência de teste:
                let base_frequency = match sfx_id {
                    0 => 220.0, // Nota Lá (A3) - Efeito de pulo/bip grave
                    1 => 440.0, // Nota Lá (A4) - Bip padrão de clique
                    2 => 880.0, // Nota Lá (A5) - Moeda/Item coletado (Agudo)
                    _ => (sfx_id as f32 * 10.0).clamp(100.0, 2000.0), // Frequência dinâmica baseada no ID passado
                };

                // Escolhe formas de onda dinâmicas para testes audíveis baseadas no canal escolhido
                let waveform = (channel as u8) % 4; // 0=Sine, 1=Square, 2=Triangle, 3=Sawtooth
                let duration = length.unwrap_or(150) as u32; // Duração padrão em milissegundos se omitido

                // Envia o comando via canal crossbeam para ser consumido de forma assíncrona no main.rs
                let cmd = AudioCommand::PlaySfx {
                    channel,
                    waveform,
                    notes: vec![base_frequency],
                    duration_ms: duration,
                };

                let _ = tx.send(cmd);
                Ok(())
            },
        )?,
    )?;

    let s = Rc::clone(&state);
    globals.set(
        "sfx_arpeggio", // Vamos criar uma função específica para o arpeggiator
        lua.create_function(
            move |_, (notes_table, channel_arg, duration_ms_arg): (mlua::Table, Option<i32>, Option<u32>)| {
                let state_ref = s.borrow();
                let tx = match &state_ref.audio_tx {
                    Some(t) => t,
                    None => return Ok(()),
                };

                let channel = match channel_arg {
                    Some(c) => if c < 0 { 0 } else { c as usize & 3 },
                    None => 0,
                };

                // Extrai a lista de frequências da tabela Lua
                let mut notes = Vec::new();
                for pair in notes_table.pairs::<i32, f32>() {
                    if let Ok((_, freq)) = pair {
                        notes.push(freq);
                    }
                }

                // Fallback caso a tabela venha vazia
                if notes.is_empty() {
                    notes.push(440.0);
                }

                let duration = duration_ms_arg.unwrap_or(300);
                let waveform = (channel as u8) % 4; // 0=Sine, 1=Square, 2=Triangle, 3=Sawtooth

                let cmd = AudioCommand::PlaySfx {
                    channel,
                    waveform,
                    notes, // Envia o vetor completo de notas
                    duration_ms: duration,
                };

                let _ = tx.send(cmd);
                Ok(())
            },
        )?,
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

    let s = Rc::clone(&state);
    globals.set(
        "btn",
        lua.create_function(move |_, (button_idx, player_idx): (usize, Option<usize>)| {
            let p = player_idx.unwrap_or(0) & 3;
            if button_idx < 6 {
                Ok(s.borrow().buttons[p][button_idx])
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
    globals.set("music", lua.create_function(|_, _n: i32| Ok(()))?)?;
    Ok(())
}
