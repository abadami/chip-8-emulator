use std::fs::File;
use std::io::Read;
use rand::prelude::*;

const START_ADDRESS: u16 = 0x200;
const FONT_SET_START_ADDRESS: u16 = 0x50;
const FONT_SET_SIZE: usize = 80;
const VIDEO_HEIGHT: u8 = 32;
const VIDEO_WIDTH: u8 = 64;
const VIDEO_SIZE: usize = VIDEO_HEIGHT as usize * VIDEO_WIDTH as usize;

const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    program_counter: u16,
    stack: [u16; 16],
    stack_pointer: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [u32; VIDEO_SIZE],
    opcode: u16,
    random_gen: ThreadRng,
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut emulator_data = Chip8{
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            program_counter: START_ADDRESS,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; 64 * 32],
            opcode: 0,
            random_gen: rand::rng()
        };

        for (i, data) in FONT_SET.iter().enumerate() {
            let address = FONT_SET_START_ADDRESS + i as u16;

            emulator_data.memory[address as usize] = *data
        }

        emulator_data
    }

    pub fn cycle(&mut self) {
        //fetch
        self.opcode = self.fetch();

        //decode & execute
        self.execute(self.opcode);
    }

    pub fn load_rom(&mut self, filename: &str) -> std::io::Result<()> {
        let file = File::open(filename)?;

        for (index, byte) in file.bytes().enumerate() {
            self.memory[0x200 + index] = byte.unwrap_or_default();
        }

        Ok(())
    }

    fn fetch(&mut self) -> u16 {
        let opcode = (self.memory[self.program_counter as usize] as u16) << 8 | (self.memory[(self.program_counter + 1) as usize] as u16);
        self.program_counter += 2;
        opcode
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => (),
            (0, 0, 0xE, 0) => self.cls(),
            (0, 0, 0xE, 0xE) => self.ret(),
            (1, _, _, _) => self.jp(),
            (2, _, _, _) => self.call(),
            (3, _, _, _) => self.skip_equal(),
            (4, _, _, _) => self.skip_not_equal(),
            (5, _, _, 0) => self.skip_if_variables_equal(),
            (6, _, _, _) => self.load_variable(),
            (7, _, _, _) => self.add_variable(),
            (8, _, _, 0) => self.load_variable_from_variable(),
            (8, _, _, 1) => self.or_variables(),
            (8, _, _, 2) => self.and_variables(),
            (8, _, _, 3) => self.xor_variables(),
            (8, _, _, 4) => self.add_variables(),
            (8, _, _, 5) => self.subtract_variables(),
            (8, _, _, 6) => self.right_shift(),
            (8, _, _, 7) => self.subtract_variables_n(),
            (8, _, _, 0xE) => self.shift_left(),
            (9, _, _, 0) => self.skip_next_instruction_ne(),
            (0xA, _, _, _) => self.load_index(),
            (0xB, _, _, _) => self.jump_to_location(),
            (0xC, _, _, _) => self.random_byte(),
            (0xD, _, _, _) => self.draw(),
            (0xE, _, 9, 0xE) => self.skip_next_instruction_if_key(),
            (0xE, _, 0xA, 1) => self.skip_next_instruction_if_not_key(),
            (0xF, _, 0, 0xA) => self.wait_for_keypress(),
            (0xF, _, 1, 5) => self.delay_timer(),
            (0xF, _, 1, 8) => self.sound_timer(),
            (0xF, _, 1, 0xE) => self.add_to_index(),
            (0xF, _, 2, 9) => self.load_font(),
            (0xF, _, 3, 3) => self.store_bcd_representation(),
            (0xF, _, 5, 5) => self.store_up_to_variable(),
            (0xF, _, 6, 5) => self.read_up_to_variable(),
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    fn rand_byte(&mut self) -> u8 {
        self.random_gen.random_range(0..255) as u8
    }

    // 00E0 - CLS
    fn cls(&mut self) -> () {
        self.video.fill(0);
    }

    // 00EE - RET
    fn ret(&mut self) -> () {
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    // 1nnn - JP
    fn jp(&mut self) -> () {
        let address = self.opcode & 0x0FFF;

        self.program_counter = address;
    }

    // 2nnn - CALL
    fn call(&mut self) -> () {
        let address = self.opcode & 0x0FFF;

        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = address;
    }

    // 3xkk - SE Vx, byte
    fn skip_equal(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let byte = self.opcode & 0x00FF;

        if self.registers[variable_x as usize] == byte as u8 {
            self.program_counter += 2;
        }
    }

    // 4xkk - SNE Vx, byte
    fn skip_not_equal(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let byte = self.opcode & 0x00FF;

        if self.registers[variable_x as usize] != byte as u8 {
            self.program_counter += 2;
        }
    }

    // 5xy0 - SE Vx, Vy
    fn skip_if_variables_equal(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;

        if self.registers[variable_x as usize] == self.registers[variable_y as usize] {
            self.program_counter += 2;
        }
    }

    // 6xkk - LD Vx, byte
    fn load_variable(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let byte = self.opcode & 0x00FF;

        self.registers[variable_x as usize] = byte as u8;
    }

    // 7xkk - ADD Vx, byte
    fn add_variable(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let byte = self.opcode & 0x00FF;

        self.registers[variable_x as usize] += byte as u8;
    }

    // 8xy0 - LD Vx, Vy
    fn load_variable_from_variable(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;

        self.registers[variable_x as usize] = self.registers[variable_y as usize];
    }

    // 8xy1 - OR Vx, Vy
    fn or_variables(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;

        self.registers[variable_x as usize] |= self.registers[variable_y as usize];
    }

    // 8xy2 - AND Vx, Vy
    fn and_variables(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;
        self.registers[variable_x as usize] &= self.registers[variable_y as usize];
    }

    // 8xy3 - XOR Vx, Vy
    fn xor_variables(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;
        self.registers[variable_x as usize] ^= self.registers[variable_y as usize];
    }

    // 8xy4 - ADD Vx, Vy
    fn add_variables(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;

        let sum = (self.registers[variable_y as usize] as u16) + (self.registers[variable_x as usize] as u16);

        if sum > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[variable_x as usize] = (sum & 0xFF) as u8;
    }

    // 8xy5 - SUB Vx, Vy
    fn subtract_variables(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;

        if self.registers[variable_x as usize] > self.registers[variable_y as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[variable_x as usize] -= self.registers[variable_y as usize];

    }

    // 8xy6 - SHR Vx
    fn right_shift(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;

        self.registers[0xF] = self.registers[variable_x as usize] & 0x1;

        self.registers[variable_x as usize] >>= 1;
    }

    // 8xy7 - SUBN Vx, Vy
    fn subtract_variables_n(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;

        if self.registers[variable_y as usize] > self.registers[variable_x as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[variable_x as usize] -= self.registers[variable_y as usize];
    }

    // 8xyE - SHL Vx {, Vy}
    fn shift_left(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;

        self.registers[0xF] = (self.registers[variable_x as usize] & 0x80) >> 7;

        self.registers[variable_x as usize] <<= 1;
    }

    // 9xy0 - SNE Vx, Vy
    fn skip_next_instruction_ne(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;

        if self.registers[variable_y as usize] != self.registers[variable_x as usize] {
            self.program_counter += 2;
        }
    }

    // Annn - LD I, addr
    fn load_index(&mut self) -> () {
        let address = self.opcode & 0x0FFF;

        self.index = address;
    }

    // Bnnn - JP V0, addr
    fn jump_to_location(&mut self) -> () {
        let address = self.opcode & 0x0FFF;

        self.program_counter = (self.registers[0] as u16) + address;
    }

    // Cxkk - RND Vx, byte
    fn random_byte(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let byte = self.opcode & 0x00FF;

        self.registers[variable_x as usize] = self.rand_byte() & byte as u8;
    }

    // Dxyn - DRW Vx, Vy, nibble
    fn draw(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;
        let height = self.opcode & 0x0F00;

        let position_x = self.registers[variable_x as usize] % VIDEO_WIDTH;
        let position_y = self.registers[variable_y as usize] % VIDEO_HEIGHT;

        self.registers[0xF] = 0;

        for row in 0..height {
            let sprite_byte = self.memory[(self.index + row) as usize];

            for col in 0..8 {
                let sprite_pixel = sprite_byte & (0x80 >> col);
                let screen_pixel: &mut u32 = &mut self.video[((position_y + (row as u8)) * VIDEO_WIDTH + (position_x + col)) as usize];

                if sprite_pixel > 0 {
                    if *screen_pixel == 0xFFFFFFFF {
                        self.registers[0xF] = 1;
                    }

                    *screen_pixel ^= 0xFFFFFFFF;
                }
            }
        }
    }

    // Ex9E - SKP Vx
    fn skip_next_instruction_if_key(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let key = self.registers[variable_x as usize];

        if self.keypad[key as usize] > 0 {
            self.program_counter += 2;
        }
    }

    // ExA1 - SKNP Vx
    fn skip_next_instruction_if_not_key(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;

        let key = self.registers[variable_x as usize];

        if self.keypad[key as usize] == 0 {
            self.program_counter += 2;
        }
    }

    // Fx0A - LD Vx, K
    fn wait_for_keypress(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;

        if self.keypad[0] > 0 {
            self.registers[variable_x as usize] = 0;
        } else if self.keypad[1] > 0 {
            self.registers[variable_x as usize] = 1;
        } else if self.keypad[2] > 0 {
            self.registers[variable_x as usize] = 2;
        } else if self.keypad[3] > 0 {
            self.registers[variable_x as usize] = 3;
        } else if self.keypad[4] > 0 {
            self.registers[variable_x as usize] = 4;
        } else if self.keypad[5] > 0 {
            self.registers[variable_x as usize] = 5;
        } else if self.keypad[6] > 0 {
            self.registers[variable_x as usize] = 6;
        } else if self.keypad[7] > 0 {
            self.registers[variable_x as usize] = 7;
        } else if self.keypad[8] > 0 {
            self.registers[variable_x as usize] = 8;
        } else if self.keypad[9] > 0 {
            self.registers[variable_x as usize] = 9;
        } else if self.keypad[10] > 0 {
            self.registers[variable_x as usize] = 10;
        } else if self.keypad[11] > 0 {
            self.registers[variable_x as usize] = 11;
        } else if self.keypad[12] > 0 {
            self.registers[variable_x as usize] = 12;
        } else if self.keypad[13] > 0 {
            self.registers[variable_x as usize] = 13;
        } else if self.keypad[14] > 0 {
            self.registers[variable_x as usize] = 14;
        } else if self.keypad[15] > 0 {
            self.registers[variable_x as usize] = 15;
        } else {
            self.program_counter -= 2;
        }
    }

    // Fx15 - LD DT, Vx
    fn delay_timer(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        self.delay_timer = self.registers[variable_x as usize];
    }

    // Fx18 - LD ST, Vx
    fn sound_timer(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        self.sound_timer = self.registers[variable_x as usize];
    }

    // Fx1E - ADD I, Vx
    fn add_to_index(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        self.index += self.registers[variable_x as usize] as u16;
    }

    // Fx29 - LD F, Vx
    fn load_font(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let digit = self.registers[variable_x as usize];

        self.index = FONT_SET_START_ADDRESS + (5 * (digit as u16));
    }

    // Fx33 - LD B, Vx
    fn store_bcd_representation(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let mut value = self.registers[variable_x as usize];

        self.memory[(self.index + 2) as usize] = value % 10;
        value /= 10;

        self.memory[(self.index + 1) as usize] = value % 10;
        value /= 10;

        self.memory[self.index as usize] = value % 10;
    }

    // Fx55 - LD [I], Vx
    fn store_up_to_variable(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;

        for i in 0..=variable_x {
            self.memory[(self.index + i) as usize] = self.registers[i as usize];
        }
    }

    // Fx65 - LD Vx, [I]
    fn read_up_to_variable(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;

        for i in 0..=variable_x {
            self.registers[i as usize] = self.memory[(self.index + i) as usize];
        }
    }
}