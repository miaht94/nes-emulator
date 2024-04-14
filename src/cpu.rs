use crate::bus::{Bus, Memory};
use bitflags::bitflags;
enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
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
}

impl Cpu {
    fn new() -> Self {
        Cpu { program_counter: 0, register_a: 0, register_x: 0, register_y: 0, stack_pointer: 0, bus: Bus::new() }
    }

    fn calculate_address(&self, address_mode: &AddressingMode) -> u16 {
        match address_mode {
            &AddressingMode::Immediate => 1,
            _ => 1
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