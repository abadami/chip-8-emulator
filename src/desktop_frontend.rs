use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::sys::KeyCode;
use sdl2::video::Window;
use crate::chip8::{VIDEO_SIZE, VIDEO_WIDTH};

pub fn draw_screen(canvas: &mut Canvas<Window>, display: &[u32; VIDEO_SIZE]) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in display.iter().enumerate() {
        if *pixel > 0 {
            let x = (i % VIDEO_WIDTH as usize) as i32;
            let y = (i / VIDEO_WIDTH as usize) as i32;

            let rect = Rect::new(x * 10, y * 10, 10, 10);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}

pub fn key_to_btn(key: Keycode) -> Option<u8> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
