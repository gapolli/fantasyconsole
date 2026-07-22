// src/cart/loader.rs
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write, Error, ErrorKind};
use std::path::Path;
use crate::core::ConsoleMode;

const MAGIC_BYTES: &[u8; 4] = b"FCST"; // Fantasy Console Standard
const CHUNK_CODE: u8 = 0x01;
const CHUNK_SPRITES: u8 = 0x02;

pub struct Cartridge {
    pub lua_code: String,
    pub sprite_sheet: Vec<u8>,
}

/// Carrega o formato de texto legível clássico da PICO-8 (.p8)
pub fn load_p8_file<P: AsRef<Path>>(path: P) -> io::Result<Cartridge> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut lua_code = String::new();
    let mut sprite_sheet = vec![0u8; 256 * 256];
    let mut current_section = String::new();
    let mut gfx_row = 0;
    let mut found_sections = false;
    let mut full_text_fallback = String::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();

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

    if !found_sections {
        lua_code = full_text_fallback;
    }

    Ok(Cartridge { lua_code, sprite_sheet })
}

/// NOVO: Desempacota o formato binário nativo .fc e retorna o modo guardado no cabeçalho
pub fn deserialize_fc_file<P: AsRef<Path>>(path: P) -> io::Result<(Cartridge, ConsoleMode)> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut reader = &buffer[..];

    // 1. Validar Magic Bytes
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != MAGIC_BYTES {
        return Err(Error::new(ErrorKind::InvalidData, "Assinatura do cartucho .fc inválida"));
    }

    // 2. Ler Cabeçalho (Versão, Modo, Reservado)
    let mut header = [0u8; 4];
    reader.read_exact(&mut header)?;
    
    let _version = header[0];
    let mode = match header[1] {
        0x00 => ConsoleMode::Pico8,
        0x01 => ConsoleMode::Tic80,
        _ => return Err(Error::new(ErrorKind::InvalidData, "Modo de console desconhecido no arquivo .fc")),
    };

    let mut cartridge = Cartridge {
        lua_code: String::new(),
        sprite_sheet: vec![0u8; 256 * 256],
    };

    // 3. Loop de leitura dos Chunks Dinâmicos
    let mut chunk_type = [0u8; 1];
    while reader.read_exact(&mut chunk_type).is_ok() {
        let mut size_bytes = [0u8; 4];
        reader.read_exact(&mut size_bytes)?;
        let chunk_size = u32::from_be_bytes(size_bytes) as usize;

        let mut payload = vec![0u8; chunk_size];
        reader.read_exact(&mut payload)?;

        match chunk_type[0] {
            CHUNK_CODE => {
                cartridge.lua_code = String::from_utf8(payload)
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Código Lua corrompido no arquivo .fc"))?;
            }
            CHUNK_SPRITES => {
                // Copia os dados salvos respeitando o limite do buffer alocado
                let len = payload.len().min(cartridge.sprite_sheet.len());
                cartridge.sprite_sheet[..len].copy_from_slice(&payload[..len]);
            }
            _ => {} // Ignora chunks desconhecidos de forma segura (garante retrocompatibilidade)
        }
    }

    Ok((cartridge, mode))
}

/// NOVO: Serializa a struct Cartridge atual para o arquivo em formato .fc
pub fn serialize_fc_file<P: AsRef<Path>>(path: P, cart: &Cartridge, mode: ConsoleMode) -> io::Result<()> {
    let mut file = File::create(path)?;
    let mut buffer = Vec::new();

    // 1. Escreve Cabeçalho
    buffer.write_all(MAGIC_BYTES)?;
    let mode_byte = match mode {
        ConsoleMode::Pico8 => 0x00,
        ConsoleMode::Tic80 => 0x01,
    };
    buffer.write_all(&[0x01, mode_byte, 0x00, 0x00])?; // Versão 1, Modo, Reservado (2 bytes)

    // 2. Grava Chunk de Código
    if !cart.lua_code.is_empty() {
        buffer.write_all(&[CHUNK_CODE])?;
        let code_bytes = cart.lua_code.as_bytes();
        let size = (code_bytes.len() as u32).to_be_bytes();
        buffer.write_all(&size)?;
        buffer.write_all(code_bytes)?;
    }

    // 3. Grava Chunk de Sprites
    if !cart.sprite_sheet.is_empty() {
        buffer.write_all(&[CHUNK_SPRITES])?;
        let size = (cart.sprite_sheet.len() as u32).to_be_bytes();
        buffer.write_all(&size)?;
        buffer.write_all(&cart.sprite_sheet)?;
    }

    file.write_all(&buffer)?;
    Ok(())
}
