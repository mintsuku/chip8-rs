use std::io::{copy, Read};
use std::str::MatchIndices;
use std::task::Wake;
use std::{array, usize};
use std::fs::*;

use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;


const RAM_SIZE: usize = 4096;
const STACK_LENGTH: usize = 16;

pub struct CPU {
    pc: u16,
    index_reg: u16,
pub memory: [u8; RAM_SIZE],
    stack: [u16; 16],
    sound_timer: u8,
    delay_timer: u8,
    gprs: [u8; 17],
}




impl CPU {
    pub fn new() -> Self {
        CPU {
            pc: 0x200,
            index_reg: 0,
            memory: [0; RAM_SIZE],
            stack: [0; STACK_LENGTH],
            sound_timer: 0,
            delay_timer: 0,
            gprs: [0, 1, 2, 3, 4, 5, 6 , 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        }
    }


    pub fn load(&mut self, file: String) {
        let mut file = File::open(file).expect("Failed to open file");
        let offset = 0x200;
        let bytes = copy(&mut file, &mut &mut self.memory[offset as usize..]).unwrap();
        let _ = &self.memory[offset..][..bytes as usize];
    }


    pub fn fetch(&mut self) -> u16 {

        let instruction_bytes = &self.memory[self.pc as usize..self.pc as usize+2];
        self.pc += 2;
        let high_bits = instruction_bytes[0];
        let low_bits = instruction_bytes[1];

    
        let shifted_bytes = (high_bits as u16) << 8;
        
        let instruction = shifted_bytes as u16 | (low_bits as u16);

        instruction
    }

    pub fn execute(&mut self, instruction: u16, frame_buffer: &mut [u8]) {
        let mask = 0xF000;
        let nibble = instruction & mask;
    
        if instruction == 0x00E0 {
            println!("Instruction is clear screen!");
            // Clear the frame buffer
            for pixel in frame_buffer.iter_mut() {
                *pixel = 0;
            }
        } else if instruction == 0x00EE {
            println!("Returning from subroutine!");
        }
    
        match nibble {
            0xA000 => {
                let value = instruction & 0x0FFF;
                self.index_reg = value;
            }
            0x1000 => {
                let jmp_location = instruction & 0x0FFF;
                println!("jmp location: {}", jmp_location);
                self.pc = jmp_location;
            }
            0x6000 => {
                let vx_register = (instruction & 0x0F00) >> 8;

                println!("Instruction: {:04X}", instruction);
                println!("Vx Register Num: {:02X}", vx_register);
                let register_value = instruction & 0x00FF;
                println!("regiser: {}", register_value);
                self.set_vx(vx_register, register_value);
            }
            0x7000 => {
                println!("0x7000 Instruction: {:20X}", instruction);
                let vx_register = instruction & 0x0F00;
                println!("0x7000 register : {:02X}", vx_register >> 8);
                let value = instruction & 0x00FF;
                self.add_vx(vx_register >> 8, value);
            }
            0xD000 => {
                let vx = (instruction & 0x0F00) >> 8;
                let vy = (instruction & 0x00F0) >> 4;
                let x = self.gprs[vx as usize];
                let y = self.gprs[vy as usize];
                let n = instruction & 0x000F;
                let vf = self.draw(x, y, n as u8, frame_buffer);
                println!("Drawing");
                self.gprs[15] = vf;
            }
            _ => {}
        }
    }
    
    fn set_vx(&mut self, register: u16, value: u16) {
        self.gprs[register as usize] = value as u8;
    }
    
    fn add_vx(&mut self, register: u16, value: u16) {
        println!("{} : {}", register, value);
        self.gprs[register as usize] = self.gprs[register as usize & 0x000F].wrapping_add(value as u8);
    }
    
    fn draw(&mut self, x: u8, y: u8, n: u8, frame_buffer: &mut [u8]) -> u8 {
        let mut vf = 0;
        let mem = self.read_bytes(self.index_reg as usize, n as usize);
    
        if let Some(arr) = mem {
            for (row, &byte) in arr.iter().enumerate() {
                for col in 0..8 {
                    let pixel = (byte >> (7 - col)) & 1;
                    let fb_index = ((y as usize + row) * 64 + (x as usize + col)) as usize;
                    if pixel == 1 {
                        if frame_buffer[fb_index] == 1 {
                            vf = 1;
                        }
                        frame_buffer[fb_index] ^= 1;
                    }
                }
            }
        }
    
        vf
    }

    fn read_bytes(&mut self, offset: usize, length: usize) -> Option<&[u8]> {
        if offset + length <= RAM_SIZE {
            Some(&self.memory[offset..offset + length])
        } else {
            None 
        }
    }
}
