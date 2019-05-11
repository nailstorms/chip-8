extern crate rand;

use vm::Vm;
use rand::Rng;

// 00E0 =
// Clear the screen.
pub fn cls(vm: &mut Vm) {
    vm.screen = [0; 64 * 32];
    vm.draw_flag = true;
    vm.pc += 2;
}

// 00EE =
// Return from subroutine.
pub fn ret(vm: &mut Vm) {
    vm.sp -= 1;
    vm.pc = vm.stack[vm.sp] as usize;
    // vm.pc += 2;
}

// 1NNN =
// Jump to address NNN.
pub fn jp_addr(vm: &mut Vm) {
    vm.pc = (vm.opcode & 0x0FFF) as usize;
}

// 2NNN =
// Call subroutine at NNN.
pub fn call_addr(vm: &mut Vm) {
    vm.stack[vm.sp] = (vm.pc) as u16;
    vm.sp += 1;
    vm.pc = (vm.opcode & 0x0FFF) as usize;
}

// 3XNN =
// Skip the next instruction if VX equals NN.
pub fn se_vx_byte(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];

    if vx == (vm.opcode & 0x00FF) as u8 {
        vm.pc += 4;
    } else {
        vm.pc += 2;
    }
}

// 4XNN =
// Skip the next instruction if VX isn't equal to NN.
pub fn sne_vx_byte(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];

    if vx != (vm.opcode & 0x00FF) as u8 {
        vm.pc += 4;
    } else {
        vm.pc += 2;
    }
}

// 5XY0 =
// Skip the next instruction if VX equals VY.
pub fn se_vx_vy(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];
    let y = ((vm.opcode & 0x00F0) >> 4) as usize;
    let vy = vm.v[y];

    if vx == vy {
        vm.pc += 4;
    } else {
        vm.pc += 2;
    }
}

// 6XNN =
// Set VX to NN.
pub fn ld_vx_byte(vm: &mut Vm) {
    vm.v[((vm.opcode & 0x0F00) >> 8) as usize] = (vm.opcode & 0x00FF) as u8;
    vm.pc += 2;
}

// 7XNN =
// Add NN to VX.
pub fn add_vx_byte(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    vm.v[x] = vm.v[x].wrapping_add((vm.opcode & 0x00FF) as u8);

    vm.v[x] = sum as u8;
    vm.pc += 2;
}

// 8XY0 =
// Set VX to the value of VY
pub fn ld_vx_vy(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let y = ((vm.opcode & 0x00F0) >> 4) as usize;

    vm.v[x] = vm.v[y];
    vm.pc += 2;
}

// 8XY1 =
// Set VX to VX or VY.
pub fn or_vx_vy(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];
    let y = ((vm.opcode & 0x00F0) >> 4) as usize;
    let vy = vm.v[y];

    vm.v[x] = vx | vy;
    vm.pc += 2;
}

// 8XY2 =
// Set VX to VX and VY.
pub fn and_vx_vy(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];
    let y = ((vm.opcode & 0x00F0) >> 4) as usize;
    let vy = vm.v[y];

    vm.v[x] = vx & vy;
    vm.pc += 2;
}

// 8XY3 =
// Set VX to VX xor VY.
pub fn xor_vx_vy(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];
    let y = ((vm.opcode & 0x00F0) >> 4) as usize;
    let vy = vm.v[y];

    vm.v[x] = vx ^ vy;
    vm.pc += 2;
}

// 8XY4 =
// Adds VY to VX. VF is set to 1 when there's a carry,
// and to 0 when there isn't.
pub fn add_vx_vy(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];
    let y = ((vm.opcode & 0x00F0) >> 4) as usize;
    let vy = vm.v[y];

    if vy > (0xFF - vx) {
        vm.v[0xF] = 1;
    } else {
        vm.v[0xF] = 0;
    }
    vm.v[x] = vx.wrapping_add(vy);
    vm.pc += 2;
}

// 8XY5 =
// Subtracts VY from VX. VF is set to 1 when there's no borrow,
// and to 0 when there is. Result is stored in VX.
pub fn sub_vx_vy(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];
    let y = ((vm.opcode & 0x00F0) >> 4) as usize;
    let vy = vm.v[y];

    if vx >= vy {
        vm.v[0xF] = 1;
    } else {
        vm.v[0xF] = 0;
    }
    vm.v[x] = vx.wrapping_sub(vy);
    vm.pc += 2;
}

// 8XY6 =
// Shifts VX right by one. VF is set to the value of
// the least significant bit of VX before the shift
pub fn shr_vx_vy(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];

    vm.v[0xF] = vx & 0x01;
    vm.v[x] >>= 1;
    vm.pc += 2;
}

// 8XY7 =
// Subtracts VX from VY. VF is set to 1 when there's no borrow,
// and to 0 when there is. Result is stored in VX.
pub fn subn_vx_vy(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];
    let y = ((vm.opcode & 0x00F0) >> 4) as usize;
    let vy = vm.v[y];

    if vy > vx {
        vm.v[0xF] = 1;
    } else {
        vm.v[0xF] = 0;
    }
    vm.v[x] = vy.wrapping_sub(vx);
    vm.pc += 2;
}

// 8XYE =
// Shifts VX left by one. VF is set to the value of
// the most significant bit of VX before the shift
pub fn shl_vx_vy(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];

    vm.v[0xF] = (vx & 0x80) >> 7;
    vm.v[x] <<= 1;
    vm.pc += 2;
}

// 9XY0 =
// Skip the next instruction if VX isn't equal to VY.
pub fn sne_vx_vy(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];
    let y = ((vm.opcode & 0x00F0) >> 4) as usize;
    let vy = vm.v[y];

    if vx != vy {
        vm.pc += 4;
    } else {
        vm.pc += 2;
    }
}

// ANNN =
// Set I to the address NNN.
pub fn ld_i_addr(vm: &mut Vm) {
    vm.i = vm.opcode & 0x0FFF;
    vm.pc += 2;
}

// BNNN =
// Jump to the address NNN plus V0.
pub fn jp_v0_addr(vm: &mut Vm) {
    vm.pc = (vm.opcode & 0x0FFF) + (vm.v[0x0] as u16);
    // vm.pc += 2;
}

// CXNN =
// Set VX to a random number, masked by NN.
pub fn rnd_vx_byte(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let mask = vm.opcode & 0x00FF;
    let random_number = rand::thread_rng().gen::<u8>();

    vm.v[x] = random_number & (mask as u8);
    vm.pc += 2;
}

// DXYN =
// Display n-byte sprite starting at memory location I at (Vx, Vy),
// Set VF = collision.
pub fn drw_vx_vy_n(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];
    let y = ((vm.opcode & 0x00F0) >> 4) as usize;
    let vy = vm.v[y];
    let height = vm.opcode & 0x000F;

    vm.v[0xF] = 0;
    for y_line in 0..height {

        // get byte
        let pixel = vm.ram[(vm.i + y_line) as usize];

        // for each pixel on this line
        for x_line in 0..8 {
            // check if the current pixel will be drawn by AND-ING it to 1 - IOW
            // check if the pixel is set to 1 (This will scan through the byte,
            // one bit at the time)

            if (pixel & (0x80 >> x_line)) != 0 {
                let current_position = ((vx as u16 + x_line as u16) + ((vy as u16 + y_line) * 64)) % (32 * 64);

                // since the pixel will be drawn, check the destination location in
                // gfx for collision (verify if that location is flipped on (== 1))

                if vm.screen[current_position as usize] == 1 {
                    vm.v[0xF] = 1; // register the collision
                }
                vm.display[current_position as usize] ^= 1;
            }
        }
    }

    vm.draw_flag = true;
    vm.pc += 2;
}

// EX9E =
// Skips the next instruction if the key stored in VX
// is pressed.
pub fn skp_vx(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let key = vm.v[x];

    if vm.key_states[key as usize] == true {
        vm.pc += 4;
    } else {
        vm.pc += 2;
    }
}

// EXA1 =
// Skip the next instruction if the key stored in VX
// isn't pressed.
pub fn sknp_vx(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let key = vm.v[x];

    if vm.key_states[key as usize] == false {
        vm.pc += 4;
    } else {
        vm.pc += 2;
    }
}

// FX07 =
// Set VX to the value of the delay timer
pub fn ld_vx_dt(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    vm.v[((vm.opcode & 0x0F00 ) >> 8) as usize] = vm.delay_timer;
    vm.pc += 2;
}

// FX0A =
// A key press is awaited, and then stored in VX.
pub fn ld_vx_k(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;

    for i in 0..16 {
        if vm.key_states[i] == true {
            vm.v[x] = i as u8;
            vm.pc += 2;
            break;
        }
    }
}

// FX15 =
// Set the delay timer to VX.
pub fn ld_dt_vx(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    vm.delay_timer = vm.v[x];
    vm.pc += 2;
}

// FX18 =
// Set the sound timer to VX.
pub fn ld_st_vx(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    vm.sound_timer = vm.v[x];
    vm.pc += 2;
}

// FX1E =
// Add VX to I.
pub fn add_i_vx(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];

    vm.i = vm.i.wrapping_add(vx as u16);
    vm.pc += 2;
}

// FX29 =
// Set I to the location of the sprite for the character in V.
pub fn ld_f_vx(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];

    // each character contains 5 elements (reason for 0x5)
    vm.i = (vx * 0x5) as u16;
    vm.pc += 2;
}

// FX33 =
// Store binary-coded decimal representation of a value
// contained in VX to addr i, i+1, and i+2.
pub fn ld_b_vx(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;
    let vx = vm.v[x];

    vm.ram[vm.i as usize] = vx / 100;
    vm.ram[(vm.i + 1) as usize] = (vx / 10) % 10;
    vm.ram[(vm.i + 2) as usize] = (vx % 100) % 10;
    vm.pc += 2;
}

// FX55 =
// Store values contained in V0-VX in memory
// starting at address I.
pub fn ld_i_vx(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;

    for index in 0..x + 1 {
        vm.ram[vm.i + index] = vm.v[index];
    }
    vm.pc += 2;
}

// FX65 =
// Fills V0-VX with values from memory
// starting at address I.
pub fn ld_vx_i(vm: &mut Vm) {
    let x = ((vm.opcode & 0x0F00) >> 8) as usize;

    for index in 0..x + 1 {
        vm.v[index] = vm.ram[vm.i + index];
    }
    vm.pc += 2;
}