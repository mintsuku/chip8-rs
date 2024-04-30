use std::fmt::format;
use std::io::Read;
use std::fs::File;

const RAM_SIZE: usize = 4096;
const STACK_LENGTH: usize = 16;

pub struct CPU {
    pub pc: u16,
    pub prev_pc: u16,
    pub index_reg: u16,
    pub memory: [u8; RAM_SIZE],
    pub stack: [u16; STACK_LENGTH],
    pub stack_pointer: u8,
    pub sound_timer: u8,
    pub delay_timer: u8,
    pub gprs: [u8; 16],
    pub key_pressed: Option<u8>,
    pub quirk_shift: bool,
    pub clipping: bool,
}

impl CPU {
    pub fn new(quirk_shift: bool, clipping: bool) -> Self {
        CPU {
            pc: 0x200,
            prev_pc: 0x200,
            index_reg: 0,
            memory: [0; RAM_SIZE],
            stack: [0; STACK_LENGTH],
            sound_timer: 0,
            stack_pointer: 0,
            delay_timer: 0,
            gprs: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            key_pressed: None,
            quirk_shift,
            clipping
        }
    }

    pub fn load(&mut self, file_name: &str) {
        let mut file = File::open(format!("roms/{}", file_name)).expect("Failed to open file");
        let offset = 0x200;
        let bytes_read = file.by_ref().take((RAM_SIZE - offset).try_into().unwrap()).read(&mut self.memory[offset..]).unwrap();
        println!("Loaded {} bytes into memory", bytes_read);
    }

    pub fn fetch(&mut self) -> u16 {
        let instruction_bytes = &self.memory[self.pc as usize..self.pc as usize + 2];
        let high_bits = instruction_bytes[0];
        let low_bits = instruction_bytes[1];
        let shifted_bytes = (high_bits as u16) << 8;
        let instruction = shifted_bytes as u16 | (low_bits as u16);

        // Check if the previous instruction was a skip instruction
        self.pc += 2;

        instruction
    }

    pub fn execute(&mut self, instruction: u16, frame_buffer: &mut [u8]) {
        match instruction {
            0x00E0 => clear_screen(frame_buffer),
            0x00EE => self.return_from_subroutine(),
            _ => {
                let nibble = instruction & 0xF000;
                self.execute_instruction(nibble, instruction, frame_buffer);
            }
        }
    }

    fn execute_instruction(&mut self, nibble: u16, instruction: u16, frame_buffer: &mut [u8]) {
        match nibble {
            0x2000 => self.call_subroutine(instruction),
            0x3000 => self.skip_if_vx_equal(instruction),
            0x4000 => self.skip_if_vx_not_equal(instruction),
            0x5000 => self.skip_if_vx_vy_equal(instruction),
            0x6000 => self.set_vx_register(instruction),
            0x8000 => self.execute_8xy_instruction(instruction),
            0x9000 => self.skip_if_vx_vy_not_equal(instruction),
            0xA000 => self.index_reg = instruction & 0x0FFF,
            0xB000 => self.jump_with_offset(instruction),
            0xE000 => self.execute_ex_instruction(instruction),
            0xF000 => self.execute_fx_instruction(instruction),
            0x1000 => self.pc = instruction & 0x0FFF,
            0x7000 => self.add_to_vx_register(instruction),
            0xD000 => self.draw(instruction, frame_buffer),
            _ => println!("Invalid instruction: {instruction:#06X} At PC Location: {0:#06X}", self.pc),
        }
    }

    fn execute_ex_instruction(&mut self, instruction: u16) {
        let vx_register = (instruction & 0x0F00) >> 8;
        let key_value = self.gprs[vx_register as usize];

        match instruction & 0x00FF {
            0x009E => {
                if let Some(pressed_key) = self.key_pressed {
                    if pressed_key == key_value {
                        self.pc += 2;
                    }
                }
            }
            0x00A1 => {
                if let Some(pressed_key) = self.key_pressed {
                    if pressed_key != key_value {
                        self.pc += 2;
                    }
                } else {
                    self.pc += 2;
                }
            }
            _ => println!("Invalid instruction: {instruction:#06X} At PC Location: {:#06X}", self.pc),
        }
    }

    pub fn execute_fx_instruction(&mut self, instruction: u16) {
        let vx_register = (instruction & 0x0F00) >> 8;
        let operation = instruction & 0x00FF;
    
        match operation {
            0x07 => self.gprs[vx_register as usize] = self.delay_timer, // FX07: Set VX to the current value of the delay timer
            0x15 => self.delay_timer = self.gprs[vx_register as usize], // FX15: Set the delay timer to the value in VX
            0x18 => self.sound_timer = self.gprs[vx_register as usize], // FX18: Set the sound timer to the value in VX
            0x1E => self.add_to_index_register(vx_register),
            0x33 => self.store_bcd(vx_register),
            0x55 => self.save_registers_to_memory(vx_register),
            0x65 => self.load_registers_from_memory(vx_register),
            0x0A => {
                if let Some(key_value) = self.key_pressed {
                    self.gprs[vx_register as usize] = key_value;
                } else {
                    self.pc -= 2; // Decrement PC to wait for a key press
                }
            }
            _ => println!("Invalid instruction: {instruction:#06X} At PC Location: {:#06X}", self.pc),
        }
    }

    fn load_registers_from_memory(&mut self, vx_register: u16) {
        let start_index = self.index_reg as usize;
        for i in 0..=vx_register {
            self.gprs[i as usize] = self.memory[start_index + i as usize];
        }
    }

    fn save_registers_to_memory(&mut self, vx_register: u16) {
        let start_index = self.index_reg as usize;
        for i in 0..=vx_register {
            self.memory[start_index + i as usize] = self.gprs[i as usize];
        }
    }

    fn store_bcd(&mut self, vx_register: u16) {
        let value = self.gprs[vx_register as usize];
        let hundreds = value / 100;
        let tens = (value / 10) % 10;
        let ones = value % 10;

        self.memory[self.index_reg as usize] = hundreds;
        self.memory[self.index_reg as usize + 1] = tens;
        self.memory[self.index_reg as usize + 2] = ones;
    }

    fn add_to_index_register(&mut self, vx_register: u16) {
        self.index_reg = self.index_reg.wrapping_add(self.gprs[vx_register as usize] as u16);
    }

    fn jump_with_offset(&mut self, instruction: u16) {
        let nnn = instruction & 0x0FFF;
        let v0 = self.gprs[0] as u16;
        self.pc = nnn + v0;
    }

    pub fn execute_8xy_instruction(&mut self, instruction: u16) {
        let vx_register = (instruction & 0x0F00) >> 8;
        let vy_register = (instruction & 0x00F0) >> 4;
        let operation = instruction & 0x000F;

        match operation {
            0x0000 => self.gprs[vx_register as usize] = self.gprs[vy_register as usize], // 8XY0: Set VX to VY
            0x0001 => self.gprs[vx_register as usize] |= self.gprs[vy_register as usize], // 8XY1: Binary OR
            0x0002 => self.gprs[vx_register as usize] &= self.gprs[vy_register as usize], // 8XY2: Binary AND
            0x0003 => self.gprs[vx_register as usize] ^= self.gprs[vy_register as usize], // 8XY3: Logical XOR
            0x0004 => {
                let (result, overflow) = self.gprs[vx_register as usize]
                    .overflowing_add(self.gprs[vy_register as usize]);
                self.gprs[vx_register as usize] = result;
                self.gprs[0xF] = overflow as u8;
            } // 8XY4: Add with carry
            0x0005 => {
                let (result, borrow) = self.gprs[vx_register as usize]
                    .overflowing_sub(self.gprs[vy_register as usize]);
                self.gprs[vx_register as usize] = result;
                self.gprs[0xF] = !borrow as u8;
            } // 8XY5: Subtract VY from VX
            0x0007 => {
                let (result, borrow) = self.gprs[vy_register as usize]
                    .overflowing_sub(self.gprs[vx_register as usize]);
                self.gprs[vx_register as usize] = result;
                self.gprs[0xF] = !borrow as u8;
            } // 8XY7: Subtract VX from VY
            0x0006 => {
                if self.quirk_shift {
                    // CHIP-48 and SUPER-CHIP behavior
                    let vx_value = self.gprs[vx_register as usize];
                    self.gprs[vx_register as usize] = vx_value >> 1;
                    self.gprs[0xF] = vx_value & 0x01;
                } else {
                    // COSMAC VIP behavior
                    self.gprs[vx_register as usize] = self.gprs[vy_register as usize];
                    let vx_value = self.gprs[vx_register as usize];
                    self.gprs[vx_register as usize] = vx_value >> 1;
                    self.gprs[0xF] = vx_value & 0x01;
                }
            } // 8XY6: Shift VX right
            0x000E => {
                if self.quirk_shift {
                    // CHIP-48 and SUPER-CHIP behavior
                    let vx_value = self.gprs[vx_register as usize];
                    self.gprs[vx_register as usize] = vx_value << 1;
                    self.gprs[0xF] = (vx_value >> 7) & 0x01;
                } else {
                    // COSMAC VIP behavior
                    self.gprs[vx_register as usize] = self.gprs[vy_register as usize];
                    let vx_value = self.gprs[vx_register as usize];
                    self.gprs[vx_register as usize] = vx_value << 1;
                    self.gprs[0xF] = (vx_value >> 7) & 0x01;
                }
            } // 8XYE: Shift VX 
            _ => println!("Invalid instruction: {instruction:#06X} At PC Location: {0:#06X}", self.pc),
        }
    }

    fn skip_if_vx_equal(&mut self, instruction: u16) {
        let vx_register = (instruction & 0x0F00) >> 8;
        let value = instruction & 0x00FF;
        if self.gprs[vx_register as usize] == value as u8 {
            self.pc += 2; // Increment pc by 2 to skip the next instruction
        }
    }

    fn skip_if_vx_not_equal(&mut self, instruction: u16) {
        let vx_register = (instruction & 0x0F00) >> 8;
        let value = instruction & 0x00FF;
        if self.gprs[vx_register as usize] != value as u8 {
            self.pc += 2; // Increment pc by 2 to skip the next instruction
        }
    }

    fn skip_if_vx_vy_equal(&mut self, instruction: u16) {
        let vx_register = (instruction & 0x0F00) >> 8;
        let vy_register = (instruction & 0x00F0) >> 4;
        if self.gprs[vx_register as usize] == self.gprs[vy_register as usize] {
            self.pc += 2; // Increment pc by 2 to skip the next instruction
        }
    }

    fn skip_if_vx_vy_not_equal(&mut self, instruction: u16) {
        let vx_register = (instruction & 0x0F00) >> 8;
        let vy_register = (instruction & 0x00F0) >> 4;
        if self.gprs[vx_register as usize] != self.gprs[vy_register as usize] {
            self.pc += 2; // Increment pc by 2 to skip the next instruction
        }
    }

    fn set_vx_register(&mut self, instruction: u16) {
        let vx_register = (instruction & 0x0F00) >> 8;
        let register_value = instruction & 0x00FF;
        self.gprs[vx_register as usize] = register_value as u8;
    }

    fn add_to_vx_register(&mut self, instruction: u16) {
        let vx_register = (instruction & 0x0F00) >> 8;
        let value = instruction & 0x00FF;
        self.gprs[vx_register as usize] = self.gprs[vx_register as usize].wrapping_add(value as u8);
    }

    fn draw(&mut self, instruction: u16, frame_buffer: &mut [u8]) {
        let vx = (instruction & 0x0F00) >> 8;
        let vy = (instruction & 0x00F0) >> 4;
        let x = self.gprs[vx as usize] as usize % 64;
        let y = self.gprs[vy as usize] as usize % 32;
        let n = instruction & 0x000F;
        let clipping = self.clipping;
        
        let mut vf = 0;
        if let Some(sprite_bytes) = self.read_bytes(self.index_reg as usize, n as usize) {
            for (row, &byte) in sprite_bytes.iter().enumerate() {
                for col in 0..8 {
                    let sprite_pixel = (byte >> (7 - col)) & 1;
                    let fb_x = x + col;
                    let fb_y = y + row;
                    
                    if clipping && (fb_y >= 32 || fb_x >= 64) {
                        continue;
                    }
                    
                    let fb_index = fb_y * 64 + fb_x;
                    let display_pixel = frame_buffer[fb_index];
                    
                    if sprite_pixel == 1 {
                        if display_pixel == 1 {
                            vf = 1;
                        }
                        frame_buffer[fb_index] ^= 1;
                    }
                }
            }
        }
        self.gprs[15] = vf;
    }

    fn read_bytes(&mut self, offset: usize, length: usize) -> Option<&[u8]> {
        if offset + length <= RAM_SIZE {
            Some(&self.memory[offset..offset + length])
        } else {
            None
        }
    }

    fn call_subroutine(&mut self, instruction: u16) {
        let address = instruction & 0x0FFF;
        self.stack[self.stack_pointer as usize] = self.pc;
        self.stack_pointer = (self.stack_pointer + 1) % STACK_LENGTH as u8;
        self.pc = address;
    }

    fn return_from_subroutine(&mut self) {
        self.stack_pointer = (self.stack_pointer + STACK_LENGTH as u8 - 1) % STACK_LENGTH as u8;
        self.pc = self.stack[self.stack_pointer as usize];
        println!("returned");
    }
}

fn clear_screen(frame_buffer: &mut [u8]) {
    println!("Instruction is clear screen!");
    frame_buffer.iter_mut().for_each(|pixel| *pixel = 0);
}