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
    let mut sprite_sheet = vec![0u8; 128 * 128];
    let mut current_section = String::new();
    let mut gfx_row = 0;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();

        if trimmed.starts_with("__") && trimmed.ends_with("__") {
            current_section = trimmed.to_string();
            continue;
        }

        match current_section.as_str() {
            "__lua__" => {
                lua_code.push_str(&line);
                lua_code.push('\n');
            }
            "__gfx__" => {
                if gfx_row < 128 && !trimmed.is_empty() {
                    // Preenche a planilha usando os caracteres disponíveis na linha do arquivo
                    for (col, chars) in trimmed.chars().enumerate().take(128) {
                        if let Some(color_idx) = chars.to_digit(16) {
                            sprite_sheet[gfx_row * 128 + col] = color_idx as u8;
                        }
                    }
                    gfx_row += 1;
                }
            }
            _ => {}
        }
    }

    Ok(Cartridge { lua_code, sprite_sheet })
}
