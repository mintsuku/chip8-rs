mod cpu;
use cpu::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator};
use sdl2::video::WindowContext;

use crate::cpu::cpu::CPU;

fn create_texture_from_array<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    width: u32,
    height: u32,
    pixel_data: &'a [u8],
) -> Result<Texture<'a>, String> {
    let mut texture = texture_creator
        .create_texture_static(PixelFormatEnum::RGB24, width, height)
        .map_err(|e| e.to_string())?;

    texture
        .update(None, pixel_data, (width * 3) as usize)
        .map_err(|e| e.to_string())?;

    Ok(texture)
}

fn main() -> Result<(), String> {
    let mut cpu = CPU::new(false, true);
    cpu.load("5-quirks.ch8");
    println!("{:02X?}", cpu.memory);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("CHIP-8 Emulator", 640, 320)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let width = 64;
    let height = 32;
    let scale = 10;
    let screen_width = width * scale;
    let screen_height = height * scale;

    let mut frame_buffer = vec![0; (width * height) as usize];

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        // Update timers
        if cpu.delay_timer > 0 {
            cpu.delay_timer -= 1;
        }
        if cpu.sound_timer > 0 {
            // Play beep sound
            cpu.sound_timer -= 1;
        }
    
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    let key_value = match keycode {
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
                    };
                    cpu.key_pressed = key_value;
                }
                Event::KeyUp { .. } => {
                    cpu.key_pressed = None;
                }
                _ => {}
            }
        }
    
        // Execute instructions only if not waiting for a key press
        
        let next_instruction = cpu.fetch();
        cpu.execute(next_instruction, &mut frame_buffer);

  

        let mut pixel_data = vec![0; (screen_width * screen_height * 3) as usize];
        for y in 0..height {
            for x in 0..width {
                let color = if frame_buffer[y * width + x] == 1 {
                    (255, 255, 255) 
                } else {
                    (0, 0, 0)
                };
                for i in 0..scale {
                    for j in 0..scale {
                        let index = (((y * scale + i) * screen_width + (x * scale + j)) * 3) as usize;
                        pixel_data[index] = color.0;
                        pixel_data[index + 1] = color.1;
                        pixel_data[index + 2] = color.2;
                    }
                }
            }
        }

        // Create a texture from the pixel_data
        let texture = create_texture_from_array(
            &texture_creator,
            screen_width.try_into().unwrap(),
            screen_height.try_into().unwrap(),
            &pixel_data,
        )?;
        

        canvas.clear();
        canvas.copy_ex(
            &texture,
            None,
            Rect::new(0, 0, screen_width.try_into().unwrap(), screen_height.try_into().unwrap()),
            0.0,
            None,
            false,
            false,
        )?;
        canvas.present();
    }

    Ok(())
}