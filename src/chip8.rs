use std::fs::File;
use std::io::Read;
use rand::prelude::*;

const START_ADDRESS: u16 = 0x200;
const FONTSET_START_ADDRESS: u16 = 0x50;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; 80] = [
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
    video: [u32; 64 * 32],
    opcode: u16,
    random_gen: ThreadRng,
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

        for (i, data) in FONTSET.iter().enumerate() {
            let address = FONTSET_START_ADDRESS + i as u16;

            emulator_data.memory[address as usize] = *data
        }

        emulator_data
    }

    pub fn load_rom(&mut self, filename: &str) -> std::io::Result<()> {
        let file = File::open(filename)?;

        for (index, byte) in file.bytes().enumerate() {
            self.memory[0x200 + index] = byte.unwrap_or_default();
        }

        Ok(())
    }

    fn rand_byte(&mut self) -> u8 {
        self.random_gen.random_range(0..255) as u8
    }

    fn cls(&mut self) -> () {
        self.video.fill(0);
    }

    fn ret(&mut self) -> () {
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    fn jp(&mut self) -> () {
        let address = self.opcode & 0x0FFF;

        self.program_counter = address;
    }

    fn call(&mut self) -> () {
        let address = self.opcode & 0x0FFF;

        self.stack[self.stack_pointer as usize] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = address;
    }

    fn skip_equal(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let byte = self.opcode & 0x00FF;

        if (self.registers[variable_x as usize] == byte as u8) {
            self.program_counter += 2;
        }
    }

    fn skip_not_equal(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let byte = self.opcode & 0x00FF;

        if (self.registers[variable_x as usize] != byte as u8) {
            self.program_counter += 2;
        }
    }

    fn skip_if_variables_equal(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;

        if (self.registers[variable_x as usize] == self.registers[variable_y as usize]) {
            self.program_counter += 2;
        }
    }

    fn load_variable(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let byte = self.opcode & 0x00FF;

        self.registers[variable_x as usize] = byte as u8;
    }

    fn add_variable(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let byte = self.opcode & 0x00FF;

        self.registers[variable_x as usize] += byte as u8;
    }

    fn load_variable_from_variable(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;

        self.registers[variable_x as usize] = self.registers[variable_y as usize];
    }

    fn or_variables(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;

        self.registers[variable_x as usize] |= self.registers[variable_y as usize];
    }

    fn and_variables(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;
        self.registers[variable_x as usize] &= self.registers[variable_y as usize];
    }

    fn xor_variables(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;
        self.registers[variable_x as usize] ^= self.registers[variable_y as usize];
    }

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

    fn subtract_variables(&mut self) -> () {
        let variable_x = (self.opcode & 0x0F00) >> 8;
        let variable_y = (self.opcode & 0x00F0) >> 4;

        if (self.registers[variable_x as usize] > self.registers[variable_y as usize]) {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[variable_x as usize] -= self.registers[variable_y as usize];

    }
}