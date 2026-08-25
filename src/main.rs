use std::fs::File;
use sdl2::event::Event;
use crate::chip8::Chip8;

pub mod chip8;
mod desktop_frontend;

fn main() {
    //Prep Sdl2 canvas
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip8 Emulator", 640, 320)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut _emulator = Chip8::new();

    canvas.clear();
    canvas.present();

    _emulator.load_rom("./.roms/test_opcode.ch8").expect("Could not load file");

    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'gameloop,
                _ => {}
            }
        }

        _emulator.cycle();
    }
}
