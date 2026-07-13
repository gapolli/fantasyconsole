use png::Decoder;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

pub fn load_p8_png_file<P: AsRef<Path>>(path: P) -> io::Result<(String, Vec<u8>)> {
    let file = File::open(path)?;
    let ref mut reader = BufReader::new(file);

    let decoder = Decoder::new(reader);
    let mut reader = decoder
        .read_info()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let bytes = &buf[..info.buffer_size()];

    let mut extracted_data = vec![0u8; 32768];
    let mut bit_index = 0;

    for &byte in bytes.iter() {
        if bit_index >= 32768 * 8 {
            break;
        }
        let target_byte = bit_index / 8;
        let target_bit = bit_index % 8;
        let lsb = byte & 0x03;
        extracted_data[target_byte] |= lsb << target_bit;
        bit_index += 2;
    }

    let mut sprite_sheet = vec![0u8; 128 * 128];
    for i in 0..0x2000 {
        let byte = extracted_data[i];
        let low_nibble = byte & 0x0F;
        let high_nibble = (byte >> 4) & 0x0F;
        if (i * 2) < sprite_sheet.len() {
            sprite_sheet[i * 2] = low_nibble;
            sprite_sheet[i * 2 + 1] = high_nibble;
        }
    }

    let code_data = &extracted_data[0x4300..0x7FFF];

    let lua_code = if code_data.starts_with(b"\npxa:") || code_data.starts_with(b"pxa:") {
        let mut pos = if code_data.starts_with(b"\npxa:") {
            5
        } else {
            4
        };

        let _decompressed_size = ((code_data[pos] as usize) << 8) | (code_data[pos + 1] as usize);
        pos += 4;

        let mut output_bytes: Vec<u8> = Vec::new();

        while pos < code_data.len() {
            let block_header = code_data[pos];
            pos += 1;
            if block_header == 0 {
                break;
            }

            for bit in 0..8 {
                if pos >= code_data.len() {
                    break;
                }

                if (block_header & (1 << bit)) == 0 {
                    output_bytes.push(code_data[pos]);
                    pos += 1;
                } else {
                    if pos + 1 >= code_data.len() {
                        break;
                    }
                    let b1 = code_data[pos] as usize;
                    let b2 = code_data[pos + 1] as usize;
                    pos += 2;

                    let offset = (b1 << 4) | (b2 >> 4);
                    let length = (b2 & 0x0F) + 2;

                    if offset > 0 && offset <= output_bytes.len() {
                        let start_index = output_bytes.len() - offset;
                        for i in 0..length {
                            if (start_index + i) < output_bytes.len() {
                                let match_byte = output_bytes[start_index + i];
                                output_bytes.push(match_byte);
                            }
                        }
                    }
                }
            }
        }
        String::from_utf8(output_bytes).unwrap_or_else(|_| "".to_string())
    } else {
        // Fallback: Se não estiver compactado, lê como string ASCII plana normal
        let mut lua_bytes = Vec::new();
        for &byte in code_data {
            if byte == 0 {
                break;
            }
            lua_bytes.push(byte);
        }
        String::from_utf8(lua_bytes).unwrap_or_else(|_| "".to_string())
    };
    Ok((lua_code, sprite_sheet))
}
