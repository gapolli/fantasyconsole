// src/renderer/editor.rs
use crate::vm::api::BackendState;
use std::collections::VecDeque;

pub struct SpriteEditor {
    pub selected_sprite_id: u32,
    pub current_tool: u8, // 0 = Lápis, 1 = Balde de Tinta
}

impl SpriteEditor {
    pub fn new() -> Self {
        Self {
            selected_sprite_id: 0,
            current_tool: 0,
        }
    }

    /// Executa o algoritmo iterativo Flood Fill (Balde de Tinta) restrito ao bloco 8x8 do sprite
    fn flood_fill(&self, sprite_sheet: &mut [u8], start_x: u32, start_y: u32, target_color: u8, replacement_color: u8) {
        if target_color == replacement_color {
            return;
        }

        // Limites absolutos do sprite selecionado na folha 256x256
        let sheet_base_x = (self.selected_sprite_id % 16) * 8;
        let sheet_base_y = (self.selected_sprite_id / 16) * 8;

        let mut queue = VecDeque::new();
        queue.push_back((start_x, start_y));

        while let Some((cx, cy)) = queue.pop_front() {
            let idx = (cy * 256 + cx) as usize;
            if sprite_sheet[idx] == target_color {
                sprite_sheet[idx] = replacement_color;

                // Verifica os 4 vizinhos (Cima, Baixo, Esquerda, Direita) travados no bloco 8x8
                if cx > sheet_base_x { queue.push_back((cx - 1, cy)); }
                if cx < sheet_base_x + 7 { queue.push_back((cx + 1, cy)); }
                if cy > sheet_base_y { queue.push_back((cy - 1, cy)); }
                if cy < sheet_base_y + 7 { queue.push_back((cy, cy + 1)); }
            }
        }
    }

    pub fn update_and_render(&mut self, s: &mut BackendState) {
        // 1. Renderização de Fundo e Layout Base
        s.cls(5); // Cinza Escuro
        s.rectfill(0, 0, s.target_width - 1, 12, 0); // Barra Superior Preta
        
        // Exibe o cabeçalho dinâmico com o Sprite ID atual selecionado
        let status_text = format!("SPRITE EDITOR - ID:{:03}", self.selected_sprite_id);
        s.draw_text(&status_text, 4, 4, 11);

        // Limites da Lona de Desenho
        let canvas_x0 = 8;
        let canvas_y0 = 16;
        let canvas_size = 64;
        s.rect(canvas_x0 - 1, canvas_y0 - 1, canvas_x0 + canvas_size, canvas_y0 + canvas_size, 7);

        // 2. Renderização do Grid 8x8 Expandido
        let sheet_x = (self.selected_sprite_id % 16) * 8;
        let sheet_y = (self.selected_sprite_id / 16) * 8;

        for py in 0..8 {
            for px in 0..8 {
                let pixel_x = sheet_x + px;
                let pixel_y = sheet_y + py;
                let color_idx = s.sprite_sheet[(pixel_y * 256 + pixel_x) as usize];

                let bx0 = canvas_x0 + (px as i32 * 8);
                let by0 = canvas_y0 + (py as i32 * 8);
                s.rectfill(bx0, by0, bx0 + 7, by0 + 7, color_idx);
                s.rect(bx0, by0, bx0 + 7, by0 + 7, 1); // Linhas da grade sutil
            }
        }

        // 3. Renderização da Paleta de Cores (Grid 4x4)
        let palette_x0 = 80;
        let palette_y0 = 16;
        for c in 0..16 {
            let cx = palette_x0 + (c % 4) * 10;
            let cy = palette_y0 + (c / 4) * 10;
            s.rectfill(cx, cy, cx + 8, cy + 8, c as u8);
            if s.current_color == c as u8 {
                s.rect(cx - 1, cy - 1, cx + 9, cy + 9, 7); // Borda branca na ativa
            }
        }

        // 4. Renderização dos Botões de Ferramentas (UI Lateral)
        let tool_x = 80;
        let tool_y = 62;
        // Botão Lápis (T0)
        s.rectfill(tool_x, tool_y, tool_x + 18, tool_y + 10, if self.current_tool == 0 { 11 } else { 0 });
        s.draw_text("PEN", tool_x + 3, tool_y + 3, if self.current_tool == 0 { 0 } else { 7 });
        
        // Botão Balde (T1)
        s.rectfill(tool_x + 22, tool_y, tool_x + 40, tool_y + 10, if self.current_tool == 1 { 11 } else { 0 });
        s.draw_text("BKT", tool_x + 25, tool_y + 3, if self.current_tool == 1 { 0 } else { 7 });

        // Instanciação de cliques do mouse
        if s.mouse_left_clicked {
            let mx = s.mouse_x;
            let my = s.mouse_y;

            // Caso A: Clique dentro da Lona de Desenho
            if mx >= canvas_x0 && mx < canvas_x0 + canvas_size && my >= canvas_y0 && my < canvas_y0 + canvas_size {
                let clicked_px = ((mx - canvas_x0) / 8) as u32;
                let clicked_py = ((my - canvas_y0) / 8) as u32;
                let target_x = sheet_x + clicked_px;
                let target_y = sheet_y + clicked_py;

                if self.current_tool == 0 {
                    // Lápis: Altera pixel único
                    s.sprite_sheet[(target_y * 256 + target_x) as usize] = s.current_color;
                } else if self.current_tool == 1 {
                    // Balde: Dispara o Flood Fill
                    let old_color = s.sprite_sheet[(target_y * 256 + target_x) as usize];
                    self.flood_fill(&mut s.sprite_sheet, target_x, target_y, old_color, s.current_color);
                }
            }

            // Caso B: Seleção de Cores na Paleta
            if mx >= palette_x0 && mx < palette_x0 + 40 && my >= palette_y0 && my < palette_y0 + 40 {
                let pc = ((mx - palette_x0) / 10) + ((my - palette_y0) / 10) * 4;
                if pc >= 0 && pc < 16 { s.current_color = pc as u8; }
            }

            // Caso C: Seleção de Ferramentas (Botões PEN / BKT)
            if my >= tool_y && my <= tool_y + 10 {
                if mx >= tool_x && mx <= tool_x + 18 { self.current_tool = 0; }
                if mx >= tool_x + 22 && mx <= tool_x + 40 { self.current_tool = 1; }
            }
        }

        // 5. Cursor Crosshair Retro
        let mx = s.mouse_x;
        let my = s.mouse_y;
        if mx >= 0 && mx < s.target_width && my >= 0 && my < s.target_height {
            let cursor_color = if s.current_color == 7 { 0 } else { 7 };
            s.pset(mx, my, cursor_color);
            s.pset(mx - 1, my, cursor_color);
            s.pset(mx + 1, my, cursor_color);
            s.pset(mx, my - 1, cursor_color);
            s.pset(mx, my + 1, cursor_color);
        }
    }
}

pub struct MapEditor {
    pub current_tile_id: u8,
    pub camera_x: i32,
    pub camera_y: i32,
    pub input_cooldown: u8,
}

impl MapEditor {
    pub fn new() -> Self {
        Self {
            current_tile_id: 1,
            camera_x: 0,
            camera_y: 0,
            input_cooldown: 0, // Inicializa pronto para receber input
        }
    }

    pub fn update_and_render(&mut self, s: &mut BackendState) {
        // --- BLINDAGEM DE COORDENADAS SÊNIOR ---
        // Força a VM a resetar a câmera e o recorte para o espaço absoluto da tela.
        // Isso impede que a câmera do jogo desloque os tiles do editor para fora da visão!
        s.camera_x = 0;
        s.camera_y = 0;
        s.clip_x0 = 0;
        s.clip_y0 = 0;
        s.clip_x1 = s.target_width - 1;
        s.clip_y1 = s.target_height - 1;

        // Limpa a tela com Cinza Escuro (Cor 5)
        s.cls(5);

        // Atualiza o cooldown de inputs a cada frame
        if self.input_cooldown > 0 {
            self.input_cooldown -= 1;
        }

        // Processa o scroll compassado do cursor amarelo
        if self.input_cooldown == 0 {
            let mut moved = false;

            if s.buttons[0][0] { self.camera_x = (self.camera_x - 1).max(0); moved = true; } // Esquerda
            if s.buttons[0][1] { self.camera_x = (self.camera_x + 1).min(239); moved = true; } // Direita
            if s.buttons[0][2] { self.camera_y = (self.camera_y - 1).max(0); moved = true; } // Cima
            if s.buttons[0][3] { self.camera_y = (self.camera_y + 1).min(135); moved = true; } // Baixo

            if moved {
                self.input_cooldown = 8;
            }
        }

        // 2. Renderização Dinâmica Centralizada no Cursor de Foco
        let tile_size = 8;
        let view_w = s.target_width / tile_size;
        let view_h = (s.target_height - 20) / tile_size;

        // Centraliza a janela visível do mapa em torno da célula do cursor amarelo
        let start_mx = (self.camera_x - view_w / 2).max(0).min(240 - view_w);
        let start_my = (self.camera_y - view_h / 2).max(0).min(136 - view_h);

        for my in 0..view_h {
            for mx in 0..view_w {
                let map_cell_x = start_mx + mx;
                let map_cell_y = start_my + my;

                if map_cell_x < 240 && map_cell_y < 136 {
                    let tile_id = s.mget(map_cell_x, map_cell_y);
                    let screen_draw_x = mx * tile_size;
                    let screen_draw_y = my * tile_size;

                    if tile_id != 0 {
                        s.spr_ext(tile_id as u32, screen_draw_x, screen_draw_y, false, false);
                    } else {
                        // Grade de pontos guias estáticos
                        s.pset(screen_draw_x, screen_draw_y, 1);
                    }

                    // --- DESENHO DO CURSOR AMARELO DE FOCO DE COORDENADA ---
                    if map_cell_x == self.camera_x && map_cell_y == self.camera_y {
                        s.rect(screen_draw_x, screen_draw_y, screen_draw_x + 7, screen_draw_y + 7, 10);
                    }
                }
            }
        }

        // 4. Interface Inferior Fixa HUD
        let ui_y0 = s.target_height - 20;
        s.rectfill(0, ui_y0, s.target_width - 1, s.target_height - 1, 0);
        s.line(0, ui_y0, s.target_width - 1, ui_y0, 7);

        let telemetry = format!("CELL X:{:03} Y:{:03} | TILE:{} | [Z] STAMP", self.camera_x, self.camera_y, self.current_tile_id);
        s.draw_text(&telemetry, 4, ui_y0 + 6, 11);

        // Preview do Sprite à direita da barra preta
        s.spr_ext(self.current_tile_id as u32, s.target_width - 14, ui_y0 + 6, false, false);
    }
}