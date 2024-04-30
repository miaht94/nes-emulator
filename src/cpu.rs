use std::{ops::Add, result};

use crate::bus::{Bus, Memory};
use bitflags::bitflags;
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NoneAddressing
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
        const NEGATIVE          = 0b10000000;
    }
}

const STACK: u16 = 0x100;
const STACK_RESET: u8 = 0xfd;
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

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.flags.insert(CpuFlags::ZERO);
        } else {
            self.flags.remove(CpuFlags::ZERO);
        }

        if result >> 7 == 1 {
            self.flags.insert(CpuFlags::NEGATIVE);
        } else {
            self.flags.remove(CpuFlags::NEGATIVE);
        }
    }

    fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(value);
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
            AddressingMode::NoneAddressing => panic!("Do not support this addressing mode")
        }
    }
    
    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.bus.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    fn stack_push(&mut self, data: u8) {
        self.bus.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1)
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        hi << 8 | lo
    }

    fn compare(&mut self, address_mode: &AddressingMode, to_value: u8) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        if param <= to_value {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.remove(CpuFlags::CARRY)
        }
        self.update_zero_and_negative_flags(to_value.wrapping_sub(param))
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

impl Cpu {
    fn adc(&mut self, address_mode: &AddressingMode) {
        let add_param = self.bus.mem_read(self.calculate_address(address_mode));
        self.add_to_register_a(add_param)
    }

    fn and(&mut self, address_mode: &AddressingMode) {
        let and_param =  self.bus.mem_read(self.calculate_address(address_mode));
        self.set_register_a(self.register_a & and_param);
    }

    fn asl(&mut self, address_mode: &AddressingMode) {
        let (old_value, new_value) = match address_mode {
            &AddressingMode::NoneAddressing => {
                let old_register_a = self.register_a;
                self.set_register_a(self.register_a << 1);
                (old_register_a, self.register_a)
            }
            // For case addressing mode is not none addressing
            _ => {
                let old_value = self.bus.mem_read(self.calculate_address(address_mode));
                let new_value = old_value << 1;
                self.bus.mem_write(self.calculate_address(address_mode), new_value);
                (old_value, new_value)
            }
        };
        if old_value >> 7 == 0b1 {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.remove(CpuFlags::CARRY)
        }
        self.update_zero_and_negative_flags(new_value)
    }

    fn beq(&mut self) {
        let param = self.bus.mem_read(self.program_counter);
        if self.flags.contains(CpuFlags::ZERO) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bcc(&mut self) {
        let param = self.bus.mem_read(self.program_counter);
        if !self.flags.contains(CpuFlags::CARRY) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bcs(&mut self) {
        let param = self.bus.mem_read(self.program_counter);
        if self.flags.contains(CpuFlags::CARRY) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bit(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        let result = param & self.register_a;
        if result & 0b0100_0000 == 0b0100_0000 {
            self.flags.insert(CpuFlags::OVERFLOW)
        }
        if result & 0b1000_0000 == 0b1000_0000 {
            self.flags.insert(CpuFlags::NEGATIVE)
        }
    }

    fn bmi(&mut self) {
        let param = self.bus.mem_read(self.program_counter);
        if self.flags.contains(CpuFlags::NEGATIVE) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bne(&mut self) {
        let param = self.bus.mem_read(self.program_counter);
        if !self.flags.contains(CpuFlags::ZERO) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bpl(&mut self) {
        let param = self.bus.mem_read(self.program_counter);
        if !self.flags.contains(CpuFlags::NEGATIVE) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bvc(&mut self) {
        let param = self.bus.mem_read(self.program_counter);
        if !self.flags.contains(CpuFlags::OVERFLOW) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bvs(&mut self) {
        let param = self.bus.mem_read(self.program_counter);
        if self.flags.contains(CpuFlags::OVERFLOW) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn clc(&mut self) {
        self.flags.remove(CpuFlags::CARRY)
    }

    fn cld(&mut self) {
        self.flags.remove(CpuFlags::DECIMAL_MODE)
    }

    fn cli(&mut self) {
        self.flags.remove(CpuFlags::INTERRUPT_DISABLE)
    }

    fn clv(&mut self) {
        self.flags.remove(CpuFlags::OVERFLOW)
    }

    fn cmp(&mut self, address_mode: &AddressingMode) {
        self.compare(address_mode, self.register_a)
    }

    fn cpx(&mut self, address_mode: &AddressingMode) {
        self.compare(address_mode, self.register_x)
    }

    fn cpy(&mut self, address_mode: &AddressingMode) {
        self.compare(address_mode, self.register_y)
    }

    fn dec(&mut self, address_mode: &AddressingMode) {
        let subtracted_numer = self.bus.mem_read(self.calculate_address(address_mode));
        let result = subtracted_numer.wrapping_sub(1);
        self.bus.mem_write(self.calculate_address(address_mode), result);
        self.update_zero_and_negative_flags(result)
    }

    fn dex(&mut self) {
        let subtracted_numer = self.register_x;
        let result = subtracted_numer.wrapping_sub(1);
        self.register_x = result;
        self.update_zero_and_negative_flags(result)
    }

    fn dey(&mut self) {
        let subtracted_numer = self.register_y;
        let result = subtracted_numer.wrapping_sub(1);
        self.register_y = result;
        self.update_zero_and_negative_flags(result)
    }

    fn eor(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        let result = self.register_a ^ param;
        self.set_register_a(result)
    }

    fn inc(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        let result = param.wrapping_add(1);
        self.update_zero_and_negative_flags(result)
    }

    fn inx(&mut self) {
        let param = self.register_x;
        let result = param.wrapping_add(1);
        self.register_x = result;
        self.update_zero_and_negative_flags(result)
    }

    fn iny(&mut self) {
        let param = self.register_y;
        let result = param.wrapping_add(1);
        self.register_y = result;
        self.update_zero_and_negative_flags(result)
    }

    fn jump_absolute(&mut self) {
        let mem_address = self.bus.mem_read_u16(self.program_counter);
        self.program_counter = mem_address
    }

    fn jump_indirect(&mut self) {
        let mem_address = self.bus.mem_read_u16(self.program_counter);
        // let indirect_ref = self.mem_read_u16(mem_address);
        //6502 bug mode with with page boundary:
        //  if address $3000 contains $40, $30FF contains $80, and $3100 contains $50,
        // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended
        // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000

        let indirect_ref = if mem_address & 0x00FF == 0x00FF {
            let lo = self.bus.mem_read(mem_address);
            let hi = self.bus.mem_read(mem_address & 0xFF00);
            (hi as u16) << 8 | (lo as u16)
        } else {
            self.bus.mem_read_u16(mem_address)
        };

        self.program_counter = indirect_ref; 
    }

    fn jsr(&mut self) {
        self.stack_push_u16(self.program_counter.wrapping_add(1));
        self.program_counter = self.bus.mem_read_u16(self.program_counter)
    }

    fn lda(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        self.set_register_a(param)
    }
    
    fn ldx(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        self.register_x = param;
        self.update_zero_and_negative_flags(param)
    }

    fn ldy(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        self.register_y = param;
        self.update_zero_and_negative_flags(param)
    }

    fn lsr_accumulator(&mut self, address_mode: &AddressingMode) {
        let m = self.register_a;
        if m & 1 == 1 {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.insert(CpuFlags::CARRY)
        }
        self.bus.mem_write(self.calculate_address(address_mode), m >> 1);
        self.update_zero_and_negative_flags(m >> 1)
    }

    fn lsr(&mut self, address_mode: &AddressingMode) {
        let m = self.bus.mem_read(self.calculate_address(address_mode));
        if m & 1 == 1 {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.insert(CpuFlags::CARRY)
        }
        self.bus.mem_write(self.calculate_address(address_mode), m >> 1);
        self.update_zero_and_negative_flags(m >> 1)
    }

    fn ora(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        let result = self.register_a | param;
        self.set_register_a(result)
    }

    fn pha(&mut self, address_mode: &AddressingMode) {
        self.stack_push_u16(self.program_counter)
    }

    // http://wiki.nesdev.com/w/index.php/CPU_status_flag_behavior
    fn php(&mut self, address_mode: &AddressingMode) {
        let mut flags = self.flags.clone();
        flags.insert(CpuFlags::BREAK);
        flags.insert(CpuFlags::BREAK2);
        self.stack_push(flags.bits());
    }

    fn pla(&mut self, address_mode: &AddressingMode) {
        self.set_register_a(self.stack_pop())
    }

    fn plp(&mut self, address_mode: &AddressingMode) {
        self.flags = CpuFlags::from_bits_truncate(self.stack_pop());
        self.flags.remove(CpuFlags::BREAK);
        self.flags.insert(CpuFlags::BREAK2)
    }

    fn rol(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        let old_carry = self.flags.contains(CpuFlags::CARRY);
        let old_bit_seven = param >> 7 & 1;
        if old_bit_seven == 1 {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.remove(CpuFlags::CARRY)
        }
        let result = if old_carry {
            param << 1 | 0b0000_0001
        } else {
            param << 1
        };
        self.bus.mem_write(self.calculate_address(address_mode), result);
        if result >> 7 == 1 {
            self.flags.insert(CpuFlags::NEGATIVE)
        } else {
            self.flags.remove(CpuFlags::NEGATIVE)
        }
    }

    fn rol_accumulator(&mut self, address_mode: &AddressingMode) {
        let param = self.register_a;
        let old_carry = self.flags.contains(CpuFlags::CARRY);
        let old_bit_seven = param >> 7 & 1;
        if old_bit_seven == 1 {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.remove(CpuFlags::CARRY)
        }
        let result = if old_carry {
            param << 1 | 0b0000_0001
        } else {
            param << 1
        };
        self.set_register_a(result);
    }

    fn ror(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        let old_carry = self.flags.contains(CpuFlags::CARRY);
        let old_bit_zero = param & 1;
        if old_bit_zero == 1 {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.remove(CpuFlags::CARRY)
        }
        let result = if old_carry {
            param >> 1 | 0b1000_0000
        } else {
            param >> 1 
        };
        self.bus.mem_write(self.calculate_address(address_mode), result);
        if result >> 7 == 1 {
            self.flags.insert(CpuFlags::NEGATIVE)
        } else {
            self.flags.remove(CpuFlags::NEGATIVE)
        }
    }
    fn ror_accumulator(&mut self, address_mode: &AddressingMode) {
        let param = self.register_a;
        let old_carry = self.flags.contains(CpuFlags::CARRY);
        let old_bit_zero = param & 1;
        if old_bit_zero == 1 {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.remove(CpuFlags::CARRY)
        }
        let result = if old_carry {
            param >> 1 | 0b1000_0000
        } else {
            param >> 1 
        };
        self.set_register_a(result);
    }

    fn rti(&mut self) {
        self.flags.bits = self.stack_pop();
        self.flags.remove(CpuFlags::BREAK);
        self.flags.insert(CpuFlags::BREAK2);
        self.program_counter = self.stack_pop_u16();
    }

    fn rts(&mut self) {
        self.program_counter = self.stack_pop_u16().wrapping_add(1)
    }

    fn sbc(&mut self, address_mode: &AddressingMode) {
        let data = self.bus.mem_read(self.calculate_address(address_mode));
        self.add_to_register_a(((data as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }

    fn sec(&mut self) {
        self.flags.insert(CpuFlags::CARRY)
    }
    
    fn sed(&mut self) {
        self.flags.insert(CpuFlags::DECIMAL_MODE)
    }

    fn sei(&mut self) {
        self.flags.insert(CpuFlags::INTERRUPT_DISABLE)
    }

    fn sta(&mut self, address_mode: &AddressingMode) {
        self.bus.mem_write(self.calculate_address(address_mode), self.register_a)
    }

    fn stx(&mut self, address_mode: &AddressingMode) {
        self.bus.mem_write(self.calculate_address(address_mode), self.register_x)
    }

    fn sty(&mut self, address_mode: &AddressingMode) {
        self.bus.mem_write(self.calculate_address(address_mode), self.register_y)
    }
    
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x)
    }

    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y)
    }

    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x)
    }

    fn txa(&mut self) {
        self.set_register_a(self.register_x)
    }

    fn txs(&mut self) {
        self.stack_pointer = self.register_x
    }

    fn tya(&mut self) {
        self.set_register_a(self.register_y)
    }
    /// note: ignoring decimal mode
    /// http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
    fn add_to_register_a(&mut self, data: u8) {
        let sum = self.register_a as u16
            + data as u16
            + (if self.flags.contains(CpuFlags::CARRY) {
                1
            } else {
                0
            }) as u16;
        let carry = sum > 0xff;
        if carry {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }
        let result = sum as u8;
        if (data ^ result) & (result ^ self.register_a) & 0x80 != 0 {
            self.flags.insert(CpuFlags::OVERFLOW);
        } else {
            self.flags.remove(CpuFlags::OVERFLOW)
        }
        self.set_register_a(result);
    }
}