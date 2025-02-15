use bevy::ecs::system::Resource;
use std::{path::PathBuf, usize};

#[derive(Resource)]
pub struct Chip8 {
    pub memory: [u8; 4096],
    pub registers: [u8; 16],
    pub i_register: u16,
    pub pc: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: Vec<u16>,
    pub display: [[u8; 64]; 32],
    pub keys: [bool; 16],
    pub waiting_key_opcode: u16,
}

impl Default for Chip8 {
    fn default() -> Self {
        Self {
            memory: [0; 4096],
            registers: [0; 16],
            i_register: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            stack: Vec::new(),
            display: [[0; 64]; 32],
            keys: [false; 16],
            waiting_key_opcode: 0,
        }
    }
}

impl Chip8 {
    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.memory.fill(0);
        self.registers.fill(0);
        self.i_register = 0;
        self.pc = 0x200;
        self.stack.clear();
        self.display.fill([0; 64]);
        self.keys.fill(false);
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.waiting_key_opcode = 0;
    }

    pub fn restart(&mut self) {
        self.registers.fill(0);
        self.i_register = 0;
        self.pc = 0x200;
        self.stack.clear();
        self.display.fill([0; 64]);
        self.keys.fill(false);
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.waiting_key_opcode = 0;
    }

    fn load_rom(&mut self, rom: &[u8], start_address: usize) {
        self.reset();
        let end_address = start_address + rom.len();
        if end_address > self.memory.len() {
            panic!("ROM is too large to fit in memory");
        }
        for (i, &byte) in rom.iter().enumerate() {
            self.memory[start_address + i] = byte;
        }
    }

    pub fn load_from_file(&mut self, filename: &PathBuf) {
        let buffer = std::fs::read(&filename).unwrap();
        self.load_rom(&buffer, 0x200);
    }

    pub fn get_current_opcode(&self) -> u16 {
        let byte1 = self.memory[self.pc as usize] as u16;
        let byte2 = self.memory[(self.pc + 1) as usize] as u16;
        (byte1 << 8) | byte2
    }

    fn get_opcode(&mut self) -> u16 {
        if self.pc as usize + 1 >= self.memory.len() {
            panic!("Attempted to read opcode outside of memory bounds");
        }
        if self.waiting_key_opcode > 0 {
            return self.waiting_key_opcode;
        }
        let byte1 = self.memory[self.pc as usize] as u16;
        let byte2 = self.memory[(self.pc + 1) as usize] as u16;
        self.pc += 2;
        (byte1 << 8) | byte2
    }

    pub fn execute_opcode(&mut self) {
        let opcode = self.get_opcode();
        match opcode & 0xF000 {
            0x0000 => self.handle_0xxx(opcode),
            0x1000 => self.handle_1xxx(opcode),
            0x2000 => self.handle_2xxx(opcode),
            0x3000 => self.handle_3xxx(opcode),
            0x4000 => self.handle_4xxx(opcode),
            0x5000 => self.handle_5xxx(opcode),
            0x6000 => self.handle_6xxx(opcode),
            0x7000 => self.handle_7xxx(opcode),
            0x8000 => self.handle_8xxx(opcode),
            0x9000 => self.handle_9xxx(opcode),
            0xA000 => self.handle_Axxx(opcode),
            0xB000 => self.handle_Bxxx(opcode),
            0xC000 => self.handle_Cxxx(opcode),
            0xD000 => self.handle_Dxxx(opcode),
            0xE000 => self.handle_Exxx(opcode),
            0xF000 => self.handle_Fxxx(opcode),
            _ => panic!("Unknown opcode: {:#X}", opcode),
        }
    }

    fn handle_0xxx(&mut self, opcode: u16) {
        match opcode {
            0x00E0 => self.display.fill([0; 64]),
            0x00EE => self.pc = self.stack.pop().expect("Stack underflow"),
            _ => panic!("Unknown opcode: {:#X}", opcode),
        }
    }

    fn handle_1xxx(&mut self, opcode: u16) {
        self.pc = opcode & 0x0FFF;
    }

    fn handle_2xxx(&mut self, opcode: u16) {
        self.stack.push(self.pc);
        self.pc = opcode & 0x0FFF;
    }

    fn handle_3xxx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        if self.registers[x] == nn {
            self.pc += 2;
        }
    }

    fn handle_4xxx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        if self.registers[x] != nn {
            self.pc += 2;
        }
    }

    fn handle_5xxx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.registers[x] == self.registers[y] {
            self.pc += 2;
        }
    }

    fn handle_6xxx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        self.registers[x] = nn;
    }

    fn handle_7xxx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        self.registers[x] = self.registers[x].wrapping_add(nn);
    }

    fn handle_8xxx(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let vx = self.registers[x];
        let vy = self.registers[y];

        match opcode & 0xF00F {
            0x8000 => {
                self.registers[x] = vy;
            }
            0x8001 => {
                self.registers[x] |= vy;
            }
            0x8002 => {
                self.registers[x] &= vy;
            }
            0x8003 => {
                self.registers[x] ^= vy;
            }
            0x8004 => {
                let (res, carry) = vx.overflowing_add(vy);
                self.registers[x] = res;
                self.registers[0xF] = if carry { 1 } else { 0 };
            }
            0x8005 => {
                self.registers[0xF] = if vx > vy { 1 } else { 0 };
                self.registers[x] = vx.wrapping_sub(vy);
            }
            0x8006 => {
                self.registers[0xF] = vx & 0x01;
                self.registers[x] = vx >> 1;
            }
            0x8007 => {
                let (res, carry) = vy.overflowing_sub(vx);
                self.registers[0xF] = if carry { 0 } else { 1 };
                self.registers[x] = res;
            }
            0x800E => {
                self.registers[0xF] = (vx >> 7) & 1;
                self.registers[x] = vx << 1;
            }
            _ => panic!("Unknown opcode: {:#X}", opcode),
        }
    }

    fn handle_9xxx(&mut self, opcode: u16) {
        let x = ((opcode & 0xF00) >> 8) as usize;
        let y = ((opcode & 0x0F0) >> 4) as usize;
        if self.registers[x] != self.registers[y] {
            self.pc += 2;
        }
    }

    fn handle_Axxx(&mut self, opcode: u16) {
        self.i_register = opcode & 0xFFF;
    }

    fn handle_Bxxx(&mut self, opcode: u16) {
        self.pc = self.registers[0] as u16 + (opcode & 0xFFF);
    }

    fn handle_Cxxx(&mut self, opcode: u16) {
        let x = ((opcode & 0xF00) >> 8) as usize;
        let nn = (opcode & 0x0FF) as u8;
        self.registers[x] = rand::random::<u8>() & nn;
    }

    fn handle_Dxxx(&mut self, opcode: u16) {
        let vx = self.registers[((opcode & 0xF00) >> 8) as usize];
        let vy = self.registers[((opcode & 0x0F0) >> 4) as usize];
        let n = opcode & 0x000F;
        let x = (vx as usize) % 64;
        let y = (vy as usize) % 32;
        let sprite_data =
            &self.memory[self.i_register as usize..self.i_register as usize + n as usize];
        self.registers[0xF] = 0;
        for row in 0..n {
            for col in 0..8 {
                let pixel = (sprite_data[row as usize] >> (7 - col)) & 1;
                let disp_x = (x + col as usize) % 64;
                let disp_y = (y + row as usize) % 32;
                let cur_pixel = self.display[disp_y][disp_x];
                if cur_pixel > 0 && pixel > 0 {
                    self.registers[0xF] = 1;
                }
                self.display[disp_y][disp_x] = cur_pixel ^ pixel;
            }
        }
    }

    fn handle_Exxx(&mut self, opcode: u16) {
        let x = ((opcode & 0xF00) >> 8) as usize;
        let vx = self.registers[x] as usize; //key index
        match opcode & 0xF0FF {
            0xE09E => {
                if self.keys[vx] {
                    self.pc += 2;
                }
            }
            0xE0A1 => {
                if !self.keys[vx] {
                    self.pc += 2;
                }
            }
            _ => panic!("Unknown opcode: {:#X}", opcode),
        }
    }

    fn handle_Fxxx(&mut self, opcode: u16) {
        let x = ((opcode & 0xF00) >> 8) as usize;
        let vx = self.registers[x];
        match opcode & 0xF0FF {
            0xF007 => {
                self.registers[x] = self.delay_timer;
            }
            0xF00A => {
                self.waiting_key_opcode = opcode;
                for (i, &key) in self.keys.iter().enumerate() {
                    if key {
                        self.waiting_key_opcode = 0;
                        self.registers[x] = i as u8;
                        break;
                    }
                }
            }
            0xF015 => {
                self.delay_timer = self.registers[x];
            }
            0xF018 => {
                self.sound_timer = self.registers[x];
            }
            0xF01E => {
                self.i_register += self.registers[x] as u16;
            }
            0xF029 => {
                self.i_register = self.registers[x] as u16 * 0x05;
            }
            0xF033 => {
                self.memory[self.i_register as usize] = vx / 100;
                self.memory[self.i_register as usize + 1] = (vx / 10) % 10;
                self.memory[self.i_register as usize + 2] = vx % 10;
            }
            0xF055 => {
                for i in 0..=x {
                    self.memory[self.i_register as usize + i] = self.registers[i];
                }
            }
            0xF065 => {
                for i in 0..=x {
                    self.registers[i] = self.memory[self.i_register as usize + i];
                }
            }
            _ => panic!("Unknown opcode: {:#X}", opcode),
        }
    }
}
