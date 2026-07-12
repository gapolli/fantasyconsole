use crossbeam_channel::{unbounded, Receiver};
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use std::time::{Duration, Instant};

use fantasyconsole::audio::{
    sfx::{Oscillator, Waveform},
    AudioMixer, SoundChannel,
};
use fantasyconsole::cart::load_cartridge;
use fantasyconsole::vm::api::{inject_pico8_api, AudioCommand, BackendState};

const P8_RGB: [(u8, u8, u8); 16] = [
    (0x00, 0x00, 0x00),
    (0x1D, 0x2B, 0x53),
    (0x7E, 0x25, 0x53),
    (0x00, 0x87, 0x51),
    (0xAB, 0x52, 0x36),
    (0x5F, 0x57, 0x4F),
    (0xC2, 0xC3, 0xC7),
    (0xFF, 0xF1, 0xE8),
    (0xFF, 0x00, 0x4D),
    (0xFF, 0xA3, 0x00),
    (0xFF, 0xEC, 0x27),
    (0x00, 0xE4, 0x36),
    (0x29, 0xAD, 0xFF),
    (0x83, 0x76, 0x9C),
    (0xFF, 0x77, 0xA8),
    (0xFF, 0xCC, 0xAA),
];

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Uso: fantasyconsole <arquivo.p8>".to_string());
    }
    let file_path = &args[1];

    let cartridge = load_cartridge(file_path).map_err(|e| e.to_string())?;

    let scale = 4u32;
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    let window = video_subsystem
        .window("FantasyConsole", 128 * scale, 128 * scale)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 128, 128)
        .map_err(|e| e.to_string())?;

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(512),
    };

    let mut audio_device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            let sample_rate = spec.freq as f32;
            AudioMixer {
                channels: [
                    SoundChannel {
                        osc: Oscillator::new(sample_rate),
                        active: false,
                        remaining_samples: 0,
                    },
                    SoundChannel {
                        osc: Oscillator::new(sample_rate),
                        active: false,
                        remaining_samples: 0,
                    },
                    SoundChannel {
                        osc: Oscillator::new(sample_rate),
                        active: false,
                        remaining_samples: 0,
                    },
                    SoundChannel {
                        osc: Oscillator::new(sample_rate),
                        active: false,
                        remaining_samples: 0,
                    },
                ],
            }
        })
        .map_err(|e| e.to_string())?;

    audio_device.resume();
    let (tx, rx): (
        crossbeam_channel::Sender<AudioCommand>,
        Receiver<AudioCommand>,
    ) = unbounded();

    let lua = mlua::Lua::new();
    let state = Rc::new(RefCell::new(BackendState {
        screen_buffer: vec![0u8; 128 * 128],
        sprite_sheet: cartridge.sprite_sheet,
        map_buffer: vec![0u8; 128 * 64],
        palette_map: {
            let mut p = [0u8; 16];
            for i in 0..16 {
                p[i] = i as u8;
            }
            p
        },
        debug_mode: false,
        current_color: 6,
        buttons: [false; 6],
        camera_x: 0,
        camera_y: 0,
        clip_x0: 0,
        clip_y0: 0,
        clip_x1: 127,
        clip_y1: 127,
        audio_tx: Some(tx),
    }));

    inject_pico8_api(&lua, Rc::clone(&state)).map_err(|e| e.to_string())?;
    lua.load(&cartridge.lua_code)
        .exec()
        .map_err(|e| e.to_string())?;

    if let Ok(init_fn) = lua.globals().get::<_, mlua::Function>("_init") {
        init_fn.call::<_, ()>(()).map_err(|e| e.to_string())?;
    }

    let mut event_pump = sdl_context.event_pump()?;
    let mut rgb_buffer = vec![0u8; 128 * 128 * 3];
    let mut last_time = Instant::now();
    let frame_target = Duration::from_secs_f64(1.0 / 60.0);

    'running: loop {
        let now = Instant::now();
        let frame_duration = now.duration_since(last_time);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(k), ..
                } => match k {
                    Keycode::Left => state.borrow_mut().buttons[0] = true,
                    Keycode::Right => state.borrow_mut().buttons[1] = true,
                    Keycode::Up => state.borrow_mut().buttons[2] = true,
                    Keycode::Down => state.borrow_mut().buttons[3] = true,
                    Keycode::Z => state.borrow_mut().buttons[4] = true,
                    Keycode::X => state.borrow_mut().buttons[5] = true,
                    Keycode::F5 => {
                        state.borrow().save_state(1);
                        println!("Console Salvo!");
                    }
                    Keycode::F6 => {
                        state.borrow_mut().load_state(1);
                        println!("Console Restaurado!");
                    }
                    Keycode::F12 => {
                        let mut s = state.borrow_mut();
                        s.debug_mode = !s.debug_mode;
                    }
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(k), ..
                } => match k {
                    Keycode::Left => state.borrow_mut().buttons[0] = false,
                    Keycode::Right => state.borrow_mut().buttons[1] = false,
                    Keycode::Up => state.borrow_mut().buttons[2] = false,
                    Keycode::Down => state.borrow_mut().buttons[3] = false,
                    Keycode::Z => state.borrow_mut().buttons[4] = false,
                    Keycode::X => state.borrow_mut().buttons[5] = false,
                    _ => {}
                },
                _ => {}
            }
        }

        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                AudioCommand::PlaySfx {
                    channel,
                    waveform,
                    note,
                    duration_ms,
                } => {
                    let samples_to_play = ((44100.0 * duration_ms as f32) / 1000.0) as u32;
                    audio_device.lock().channels[channel].osc.frequency = note;
                    audio_device.lock().channels[channel].osc.waveform = match waveform {
                        0 => Waveform::Sine,
                        1 => Waveform::Square,
                        2 => Waveform::Triangle,
                        _ => Waveform::Sawtooth,
                    };
                    audio_device.lock().channels[channel].remaining_samples = samples_to_play;
                    audio_device.lock().channels[channel].active = true;
                }
            }
        }

        if frame_duration >= frame_target {
            last_time = now;

            if let Ok(update_fn) = lua.globals().get::<_, mlua::Function>("_update") {
                update_fn.call::<_, ()>(()).map_err(|e| e.to_string())?;
            }
            if let Ok(draw_fn) = lua.globals().get::<_, mlua::Function>("_draw") {
                draw_fn.call::<_, ()>(()).map_err(|e| e.to_string())?;
            }

            if state.borrow().debug_mode {
                let mut s = state.borrow_mut();
                let old_clip = (s.clip_x0, s.clip_y0, s.clip_x1, s.clip_y1);
                let old_cam = (s.camera_x, s.camera_y);
                s.clip(0, 0, 128, 128);
                s.camera(0, 0);

                for y in 0..7 {
                    for x in 0..128 {
                        s.screen_buffer[y * 128 + x] = 5;
                    }
                }
                s.draw_text("FC v0.1", 2, 1, 6); // Cinza claro (cor 6)

                let current_fps = (1.0 / frame_duration.as_secs_f64()).min(60.0);
                let fps_text = format!("{:.0} FPS", current_fps);
                let text_width = (fps_text.len() as i32 * 4) - 1;
                let text_x = 126 - text_width;

                let start_bar_x = 34;
                let max_bar_x = text_x - 3;
                let bar_space = max_bar_x - start_bar_x;
                let bar_width = (((current_fps / 60.0) * bar_space as f64) as i32).max(0);
                let bar_color = if current_fps >= 55.0 { 11 } else { 8 };

                s.line(start_bar_x, 3, start_bar_x + bar_width, 3, bar_color);
                s.draw_text(&fps_text, text_x, 1, 7); // Desenha métrica alinhada à direita

                s.clip_x0 = old_clip.0;
                s.clip_y0 = old_clip.1;
                s.clip_x1 = old_clip.2;
                s.clip_y1 = old_clip.3;
                s.camera_x = old_cam.0;
                s.camera_y = old_cam.1;
            }
            {
                let s = state.borrow();
                for i in 0..(128 * 128) {
                    let (r, g, b) = P8_RGB[s.screen_buffer[i] as usize];
                    rgb_buffer[i * 3] = r;
                    rgb_buffer[i * 3 + 1] = g;
                    rgb_buffer[i * 3 + 2] = b;
                }
            }

            texture
                .update(None, &rgb_buffer, 128 * 3)
                .map_err(|e| e.to_string())?;
            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}
