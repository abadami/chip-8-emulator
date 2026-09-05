use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::chip8::Chip8;
use crate::desktop_frontend::{DesktopFrontend};

pub mod chip8;
mod desktop_frontend;

const TICKS_PER_FRAME: usize = 10;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let file_path = &args[1];

    //Prep Sdl2 canvas
    let mut desktop = DesktopFrontend::new();

    let mut _emulator = Chip8::new();

    let mut pause = false;

    let mut save_state: Option<Chip8> = None;

    desktop.clear();

    _emulator.load_rom(file_path).expect("Could not load file");

    'game_loop: loop {
        for event in desktop.get_event_pump().poll_iter() {
            match event {
                Event::Quit { .. } => break 'game_loop,
                Event::KeyDown { keycode: Some(key), .. } => {
                    if let Some(btn) = desktop_frontend::key_to_btn(key) {
                        _emulator.key_press(btn, true);
                    }

                    if key == Keycode::Space {
                        pause = !pause;
                    }

                    if key == Keycode::F5 {
                        save_state = Some(_emulator.get_current_state());
                    }

                    if key == Keycode::F8 {
                        if let Some(state) = &save_state {
                            _emulator.load_emulator_state(state);
                        }
                    }
                }
                Event::KeyUp { keycode: Some(key), .. } => {
                    if let Some(btn) = desktop_frontend::key_to_btn(key) {
                        _emulator.key_press(btn, false);
                    }
                }
                _ => {}
            }
        }

        if !pause {
            for _ in 0..TICKS_PER_FRAME {
                _emulator.cycle();
            }
            _emulator.cycle_timers();
            desktop.draw_screen(_emulator.get_display());
        }
    }
}
