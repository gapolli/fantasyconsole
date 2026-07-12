pub mod loader;
pub mod png_loader;

use std::io;
use std::path::Path;
pub use loader::Cartridge;

pub fn load_cartridge<P: AsRef<Path>>(path: P) -> io::Result<Cartridge> {
    let p = path.as_ref();
    // Identifica dinamicamente a extensão do arquivo enviado
    if let Some(ext) = p.extension() {
        if ext == "png" {
            let (lua_code, sprite_sheet) = png_loader::load_p8_png_file(p)?;
            return Ok(Cartridge { lua_code, sprite_sheet });
        }
    }
    // Caso contrário, processa como arquivo de texto .p8 padrão
    loader::load_p8_file(p)
}
