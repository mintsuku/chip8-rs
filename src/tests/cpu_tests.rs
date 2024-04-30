use crate::cpu::cpu::CPU;

#[test]
fn test_00e0_clear_screen() {
    let mut cpu = CPU::new();
    let mut frame_buffer = [1; 64 * 32];
    cpu.execute(0x00E0, &mut frame_buffer);
    assert_eq!(frame_buffer, [0; 64 * 32]);
}

#[test]
fn test_1nnn_jump() {
    let mut cpu = CPU::new();
    cpu.execute(0x1234, &mut []);
    assert_eq!(cpu.pc, 0x234);
}

#[test]
fn test_6xnn_set_vx() {
    let mut cpu = CPU::new();
    cpu.execute(0x6A42, &mut []);
    assert_eq!(cpu.gprs[10], 0x42);
}

#[test]
fn test_7xnn_add_vx() {
    let mut cpu = CPU::new();
    cpu.gprs[5] = 0x10;
    cpu.execute(0x7505, &mut []);
    assert_eq!(cpu.gprs[5], 0x15);
}

#[test]
fn test_annn_set_index_reg() {
    let mut cpu = CPU::new();
    cpu.execute(0xA123, &mut []);
    assert_eq!(cpu.index_reg, 0x123);
}

#[test]
fn test_dxyn_draw() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 5;
    cpu.gprs[1] = 5;
    cpu.index_reg = 0x200;
    cpu.memory[0x200] = 0b11100000;
    cpu.memory[0x201] = 0b00000000;
    let mut frame_buffer = [0; 64 * 32];
    cpu.execute(0xD012, &mut frame_buffer);
    assert_eq!(cpu.gprs[15], 0);
    assert_eq!(frame_buffer[5 * 64 + 5], 1);
    assert_eq!(frame_buffer[5 * 64 + 6], 1);
    assert_eq!(frame_buffer[5 * 64 + 7], 1);
}

#[test]
fn test_3xnn_skip_if_vx_equal() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0x42;
    cpu.execute(0x3042, &mut []);
    assert_eq!(cpu.pc, 0x202); // skipped next instruction

    cpu.gprs[0] = 0x00;
    cpu.pc = 0x200; // reset PC
    cpu.execute(0x3042, &mut []);
    assert_eq!(cpu.pc, 0x202); // didn't skip
}

#[test]
fn test_4xnn_skip_if_vx_not_equal() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0x42;
    cpu.execute(0x4043, &mut []);
    assert_eq!(cpu.pc, 0x202); // skipped next instruction

    cpu.gprs[0] = 0x42;
    cpu.pc = 0x200; // reset PC
    cpu.execute(0x4042, &mut []);
    assert_eq!(cpu.pc, 0x202); // didn't skip
}

#[test]
fn test_5xy0_skip_if_vx_vy_equal() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0x42;
    cpu.gprs[1] = 0x42;
    cpu.execute(0x5010, &mut []);
    assert_eq!(cpu.pc, 0x202); // skipped next instruction

    cpu.gprs[0] = 0x42;
    cpu.gprs[1] = 0x43;
    cpu.pc = 0x200; // reset PC
    cpu.execute(0x5010, &mut []);
    assert_eq!(cpu.pc, 0x202); // didn't skip
}

#[test]
fn test_9xy0_skip_if_vx_vy_not_equal() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0x42;
    cpu.gprs[1] = 0x43;
    cpu.execute(0x9010, &mut []);
    assert_eq!(cpu.pc, 0x202); // skipped next instruction

    cpu.gprs[0] = 0x42;
    cpu.gprs[1] = 0x42;
    cpu.pc = 0x200; // reset PC
    cpu.execute(0x9010, &mut []);
    assert_eq!(cpu.pc, 0x202); // didn't skip
}

#[test]
fn test_8xy0_set_vx_to_vy() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0x42;
    cpu.gprs[1] = 0x24;
    cpu.execute_8xy_instruction(0x8010);
    assert_eq!(cpu.gprs[0], 0x24);
}

#[test]
fn test_8xy1_binary_or() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0b01010101;
    cpu.gprs[1] = 0b10101010;
    cpu.execute_8xy_instruction(0x8011);
    assert_eq!(cpu.gprs[0], 0b11111111);
}

#[test]
fn test_8xy2_binary_and() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0b01010101;
    cpu.gprs[1] = 0b10101010;
    cpu.execute_8xy_instruction(0x8012);
    assert_eq!(cpu.gprs[0], 0b00000000);
}

#[test]
fn test_8xy3_logical_xor() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0b01010101;
    cpu.gprs[1] = 0b10101010;
    cpu.execute_8xy_instruction(0x8013);
    assert_eq!(cpu.gprs[0], 0b11111111);
}

#[test]
fn test_8xy4_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0xF0;
    cpu.gprs[1] = 0x10;
    cpu.execute_8xy_instruction(0x8014);
    assert_eq!(cpu.gprs[0], 0x00);
    assert_eq!(cpu.gprs[0xF], 0x01);
}

#[test]
fn test_8xy5_subtract_vy_from_vx() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0x10;
    cpu.gprs[1] = 0x20;
    cpu.execute_8xy_instruction(0x8015);
    assert_eq!(cpu.gprs[0], 0xF0);
    assert_eq!(cpu.gprs[0xF], 0x00);
}

#[test]
fn test_8xy7_subtract_vx_from_vy() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0x10;
    cpu.gprs[1] = 0x20;
    cpu.execute_8xy_instruction(0x8017);
    assert_eq!(cpu.gprs[0], 0x10);
    assert_eq!(cpu.gprs[0xF], 0x01);
}

#[test]
fn test_8xy6_shift_vx_right() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0b10101010;
    cpu.execute_8xy_instruction(0x8006);
    assert_eq!(cpu.gprs[0], 0b01010101);
    assert_eq!(cpu.gprs[0xF], 0x00);
}

#[test]
fn test_8xye_shift_vx_left() {
    let mut cpu = CPU::new();
    cpu.gprs[0] = 0b10101010;
    cpu.execute_8xy_instruction(0x800E);
    assert_eq!(cpu.gprs[0], 0b01010100);
    assert_eq!(cpu.gprs[0xF], 0x01);
}

#[test]
fn test_2nnn_call_subroutine() {
    let mut cpu = CPU::new();
    cpu.execute(0x2123, &mut []);
    assert_eq!(cpu.pc, 0x123);
    assert_eq!(cpu.stack[0], 0x200);
    assert_eq!(cpu.stack_pointer, 1);
}

#[test]
fn test_00ee_return_from_subroutine() {
    let mut cpu = CPU::new();
    cpu.stack[0] = 0x456;
    cpu.stack_pointer = 1;
    cpu.execute(0x00EE, &mut []);
    assert_eq!(cpu.pc, 0x456);
    assert_eq!(cpu.stack_pointer, 0);
}

#[test]
fn test_nested_subroutine() {
    let mut cpu = CPU::new();
    cpu.execute(0x2123, &mut []); // Call subroutine at 0x123
    assert_eq!(cpu.pc, 0x123);
    assert_eq!(cpu.stack[0], 0x200); // Return address should be 0x200
    assert_eq!(cpu.stack_pointer, 1); // Stack pointer should be incremented

    cpu.execute(0x2456, &mut []); // Call another subroutine at 0x456
    assert_eq!(cpu.pc, 0x456);
    assert_eq!(cpu.stack[1], 0x123); // Return address should be updated to 0x123
    assert_eq!(cpu.stack_pointer, 2); // Stack pointer should not change

    cpu.execute(0x00EE, &mut []);
    println!("{:02X}", cpu.stack_pointer); // Return from subroutine at 0x456
    assert_eq!(cpu.pc, 0x123); // PC should be updated to the return address
    assert_eq!(cpu.stack_pointer, 1); // Stack pointer should be decremented

    cpu.execute(0x00EE, &mut []);
    println!("PC: {:02X}", cpu.pc);
    assert_eq!(cpu.pc, 0x200); 
    assert_eq!(cpu.stack_pointer, 0); // Stack pointer should remain 0
}


#[test]
fn test_fx65_load_registers_from_memory() {
    let mut cpu = CPU::new();
    cpu.index_reg = 0x500;
    cpu.memory[0x500] = 0x12;
    cpu.memory[0x501] = 0x34;
    cpu.memory[0x502] = 0x56;

    cpu.execute_fx_instruction(0xF265); // Load registers v0 to v2 from memory

    assert_eq!(cpu.gprs[0], 0x12);
    assert_eq!(cpu.gprs[1], 0x34);
    assert_eq!(cpu.gprs[2], 0x56);
}

#[test]
fn test_fx55_save_registers_to_memory() {
    let mut cpu = CPU::new();
    cpu.index_reg = 0x600;
    cpu.gprs[0] = 0x12;
    cpu.gprs[1] = 0x34;
    cpu.gprs[2] = 0x56;
    cpu.gprs[3] = 0x78;

    cpu.execute_fx_instruction(0xF355); // Save registers v0 to v3 to memory

    assert_eq!(cpu.memory[0x600], 0x12);
    assert_eq!(cpu.memory[0x601], 0x34);
    assert_eq!(cpu.memory[0x602], 0x56);
    assert_eq!(cpu.memory[0x603], 0x78);
}

#[test]
fn test_fx33_store_bcd() {
    let mut cpu = CPU::new();
    cpu.index_reg = 0x700;
    cpu.gprs[5] = 123;

    cpu.execute_fx_instruction(0xF533); // Store BCD of v5 at index_reg, index_reg + 1, index_reg + 2

    assert_eq!(cpu.memory[0x700], 1);
    assert_eq!(cpu.memory[0x701], 2);
    assert_eq!(cpu.memory[0x702], 3);
}

#[test]
fn test_fx1e_add_to_index_register() {
    let mut cpu = CPU::new();
    cpu.index_reg = 0x700;
    cpu.gprs[5] = 10;

    cpu.execute_fx_instruction(0xF51E); // Add v5 to index_reg

    assert_eq!(cpu.index_reg, 0x70A);
}