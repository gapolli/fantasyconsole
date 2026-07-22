// src/cart/mod.rs
pub mod loader;
pub mod png_loader;

use std::io;
use std::path::Path;
pub use loader::Cartridge;
use crate::core::ConsoleMode;

pub struct LoadedCartridge {
    pub cartridge: Cartridge,
    pub mode: ConsoleMode,
}

pub fn load_cartridge<P: AsRef<Path>>(path: P) -> io::Result<LoadedCartridge> {
    let p = path.as_ref();
    
    // Identifica se o arquivo é o nosso formato nativo customizado
    if let Some(ext) = p.extension() {
        if ext == "fc" {
            let (cartridge, mode) = loader::deserialize_fc_file(p)?;
            return Ok(LoadedCartridge { cartridge, mode });
        }
    }

    // Fallback de inferência de modo antigo baseado em extensão (.tic ou .p8)
    let mode = if let Some(ext) = p.extension() {
        if ext == "tic" {
            ConsoleMode::Tic80
        } else {
            ConsoleMode::Pico8
        }
    } else {
        ConsoleMode::Pico8
    };

    // Fallback de carregamento de formatos legíveis/gráficos antigos
    let cartridge = if let Some(ext) = p.extension() {
        if ext == "png" {
            let (lua_code, sprite_sheet) = png_loader::load_p8_png_file(p)?;
            Cartridge { lua_code, sprite_sheet }
        } else {
            loader::load_p8_file(p)?
        }
    } else {
        loader::load_p8_file(p)?
    };

    Ok(LoadedCartridge { cartridge, mode })
}
