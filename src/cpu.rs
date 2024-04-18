use std::ops::Add;

use crate::bus::{Bus, Memory};
use bitflags::bitflags;
enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY
}

bitflags! {
    /// # Status Register (P) http://wiki.nesdev.com/w/index.php/Status_flags
    ///
    ///  7 6 5 4 3 2 1 0
    ///  N V _ B D I Z C
    ///  | |   | | | | +--- Carry Flag
    ///  | |   | | | +----- Zero Flag
    ///  | |   | | +------- Interrupt Disable
    ///  | |   | +--------- Decimal Mode (not used on NES)
    ///  | |   +----------- Break Command
    ///  | +--------------- Overflow Flag
    ///  +----------------- Negative Flag
    ///
    pub struct CpuFlags: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK             = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIV           = 0b10000000;
    }
}

struct Cpu {
    program_counter: u16,
    register_a: u8,
    register_x: u8,
    register_y: u8,
    stack_pointer: u8,
    // memory: [u8; 0xffff]
    bus: Bus,
    flags: CpuFlags
}

impl Cpu {
    fn new() -> Self {
        Cpu { program_counter: 0, register_a: 0, register_x: 0, register_y: 0, stack_pointer: 0, bus: Bus::new(), flags: CpuFlags::from_bits_truncate(0b100100) }
    }

    fn calculate_address(&self,address_mode: &AddressingMode) -> u16 {
        match address_mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::Absolute => self.bus.mem_read_u16(self.program_counter),
            AddressingMode::AbsoluteX => {
                let base = self.bus.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_x as u16)
            }
            AddressingMode::AbsoluteY => {
                let base = self.bus.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_y as u16)
            }
            AddressingMode::ZeroPage => self.bus.mem_read(self.program_counter) as u16,
            AddressingMode::ZeroPageX => {
                let base = self.bus.mem_read(self.program_counter);
                base.wrapping_add(self.register_x) as u16
            }
            AddressingMode::ZeroPageY => {
                let base = self.bus.mem_read(self.program_counter);
                base.wrapping_add(self.register_y) as u16
            }
            AddressingMode::IndirectX => {
                let base = self.bus.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.bus.mem_read(ptr as u16);
                let hi = self.bus.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::IndirectY => {
                let base = self.bus.mem_read(self.program_counter);
                let lo = self.bus.mem_read(base as u16);
                let hi = self.bus.mem_read(base.wrapping_add(1) as u16);
                let addr = ((hi as u16) << 8 | (lo as u16)).wrapping_add(self.register_y as u16);
                addr
            }
        }
    }

    fn load(&mut self, program: &Vec<u8>) {
        // self.memory[0x8000 ..].copy_from_slice(&program);
        // self.program_counter = 0x8000;
        let mut cur_rom_address = 0x600;
        for data in program {
            self.bus.mem_write(cur_rom_address, *data);
            cur_rom_address += 1;
        }
        self.bus.mem_write_u16(0xfffc, 0x600);
    }

    fn load_and_run(&mut self, program: &Vec<u8>) {
        self.load(program);
        self.run();
    }

    fn run(&mut self) {
        self.program_counter = 0;
        loop {
            let ops_code = self.bus.mem_read(self.program_counter);
            self.program_counter += 1;
            match ops_code {
                0x00 => todo!(),
                0xA9 => todo!(),
                _ => todo!()
            }
        }
    }
}