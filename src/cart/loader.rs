use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct Cartridge {
    pub lua_code: String,
    pub sprite_sheet: Vec<u8>,
}

pub fn load_p8_file<P: AsRef<Path>>(path: P) -> io::Result<Cartridge> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut lua_code = String::new();
    let mut sprite_sheet = vec![0u8; 256 * 256]; // Expandido para o limite máximo
    let mut current_section = String::new();
    let mut gfx_row = 0;
    let mut found_sections = false;
    let mut full_text_fallback = String::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();

        // Guarda todo o texto para o caso de ser um script puro sem seções
        full_text_fallback.push_str(&line);
        full_text_fallback.push('\n');

        if trimmed.starts_with("__") && trimmed.ends_with("__") {
            current_section = trimmed.to_string();
            found_sections = true;
            continue;
        }

        match current_section.as_str() {
            "__lua__" => {
                lua_code.push_str(&line);
                lua_code.push('\n');
            }
            "__gfx__" => {
                if gfx_row < 256 && !trimmed.is_empty() {
                    for (col, chars) in trimmed.chars().enumerate().take(256) {
                        if let Some(color_idx) = chars.to_digit(16) {
                            sprite_sheet[gfx_row * 256 + col] = color_idx as u8;
                        }
                    }
                    gfx_row += 1;
                }
            }
            _ => {}
        }
    }

    // Se não encontrou nenhuma seção padrão da PICO-8 (__lua__, etc),
    // trata o arquivo inteiro como código Lua puro (padrão para testes rápidos do TIC-80)
    if !found_sections {
        lua_code = full_text_fallback;
    }

    Ok(Cartridge { lua_code, sprite_sheet })
}
