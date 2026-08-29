use sdl2::event::Event;
use crate::chip8::Chip8;
use crate::desktop_frontend::{DesktopFrontend};

pub mod chip8;
mod desktop_frontend;

const TICKS_PER_FRAME: usize = 30;

fn main() {
    //Prep Sdl2 canvas
    let mut frontend = DesktopFrontend::new();

    let mut _emulator = Chip8::new();

   frontend.clear();

    _emulator.load_rom("./.roms/tetris.ch8").expect("Could not load file");

    'game_loop: loop {
        for event in frontend.get_event_pump().poll_iter() {
            match event {
                Event::Quit { .. } => break 'game_loop,
                Event::KeyDown { keycode: Some(key), .. } => {
                    if let Some(btn) = desktop_frontend::key_to_btn(key) {
                        _emulator.key_press(btn, true);
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

        for _ in 0..TICKS_PER_FRAME {
            _emulator.cycle();
        }
        _emulator.cycle_timers();
        frontend.draw_screen(_emulator.get_display());
    }
}
