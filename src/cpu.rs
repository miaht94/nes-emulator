use std::{collections::HashMap, ops::Add, result};

use crate::{bus::{Bus, Memory}, opscode};
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
pub struct Cpu {
    pub program_counter: u16,
    register_a: u8,
    register_x: u8,
    register_y: u8,
    stack_pointer: u8,
    // memory: [u8; 0xffff]
    pub bus: Bus,
    flags: CpuFlags
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Cpu { program_counter: 0, register_a: 0, register_x: 0, register_y: 0, stack_pointer: 0, bus, flags: CpuFlags::from_bits_truncate(0b100100) }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = STACK_RESET;
        self.flags = CpuFlags::from_bits_truncate(0b100100);
        self.program_counter = self.bus.mem_read_u16(0xfffc);
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

    pub fn load(&mut self, program: &Vec<u8>) {
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
        self.run_with_callback(|arg| {})
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F) 
    where F: FnMut(&mut Self) {
        let ref opcodes: HashMap<u8, &'static opscode::OpCode> = *opscode::OPCODES_MAP;
        loop {
            let code = self.bus.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;
            let opcode = opcodes.get( &code).expect(&format!("OpCode {:x} is not regconized", code));
            println!("opcode: {:x}\tregister_a: {:x}\tregister_x: {:x}\t register_y: {:x}\tpc: {:x}, sp: {:x}, flag: {:#8b}", code, self.register_a, self.register_x, self.register_y, self.program_counter, self.stack_pointer, self.flags.bits);
            match code {
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => {
                    self.lda(&opcode.mode);
                }

                0xAA => self.tax(),
                0xe8 => self.inx(),
                0x00 => return,

                /* CLD */ 0xd8 => self.cld(),

                /* CLI */ 0x58 => self.cli(),

                /* CLV */ 0xb8 => self.clv(),

                /* CLC */ 0x18 => self.clc(),

                /* SEC */ 0x38 => self.sec(),

                /* SEI */ 0x78 => self.sei(),

                /* SED */ 0xf8 => self.sed(),

                /* PHA */ 0x48 => self.pha(),

                /* PLA */
                0x68 => {
                    self.pla();
                }

                /* PHP */
                0x08 => {
                    self.php();
                }

                /* PLP */
                0x28 => {
                    self.plp();
                }

                /* ADC */
                0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => {
                    self.adc(&opcode.mode);
                }

                /* SBC */
                0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 => {
                    self.sbc(&opcode.mode);
                }

                /* AND */
                0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => {
                    self.and(&opcode.mode);
                }

                /* EOR */
                0x49 | 0x45 | 0x55 | 0x4d | 0x5d | 0x59 | 0x41 | 0x51 => {
                    self.eor(&opcode.mode);
                }

                /* ORA */
                0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => {
                    self.ora(&opcode.mode);
                }

                /* LSR */ 0x4a => self.lsr_accumulator(),

                /* LSR */
                0x46 | 0x56 | 0x4e | 0x5e => {
                    self.lsr(&opcode.mode);
                }

                /*ASL*/ 0x0a => self.asl_accumulator(),

                /* ASL */
                0x06 | 0x16 | 0x0e | 0x1e => {
                    self.asl(&opcode.mode);
                }

                /*ROL*/ 0x2a => self.rol_accumulator(),

                /* ROL */
                0x26 | 0x36 | 0x2e | 0x3e => {
                    self.rol(&opcode.mode);
                }

                /* ROR */ 0x6a => self.ror_accumulator(),

                /* ROR */
                0x66 | 0x76 | 0x6e | 0x7e => {
                    self.ror(&opcode.mode);
                }

                /* INC */
                0xe6 | 0xf6 | 0xee | 0xfe => {
                    self.inc(&opcode.mode);
                }

                /* INY */
                0xc8 => self.iny(),

                /* DEC */
                0xc6 | 0xd6 | 0xce | 0xde => {
                    self.dec(&opcode.mode);
                }

                /* DEX */
                0xca => {
                    self.dex();
                }

                /* DEY */
                0x88 => {
                    self.dey();
                }

                /* CMP */
                0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1 => {
                    self.compare(&opcode.mode, self.register_a);
                }

                /* CPY */
                0xc0 | 0xc4 | 0xcc => {
                    self.compare(&opcode.mode, self.register_y);
                }

                /* CPX */
                0xe0 | 0xe4 | 0xec => self.compare(&opcode.mode, self.register_x),

                /* JMP Absolute */
                0x4c => {
                    let mem_address = self.bus.mem_read_u16(self.program_counter);
                    self.program_counter = mem_address;
                }

                /* JMP Indirect */
                0x6c => {
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

                /* JSR */
                0x20 => {
                    self.stack_push_u16(self.program_counter + 2 - 1);
                    let target_address = self.bus.mem_read_u16(self.program_counter);
                    self.program_counter = target_address
                }

                /* RTS */
                0x60 => {
                    self.program_counter = self.stack_pop_u16() + 1;
                }

                /* RTI */
                0x40 => {
                    self.flags.bits = self.stack_pop();
                    self.flags.remove(CpuFlags::BREAK);
                    self.flags.insert(CpuFlags::BREAK2);

                    self.program_counter = self.stack_pop_u16();
                }

                /* BNE */
                0xd0 => {
                    self.bne();
                }

                /* BVS */
                0x70 => {
                    self.bvs();
                }

                /* BVC */
                0x50 => {
                    self.bvc();
                }

                /* BPL */
                0x10 => {
                    self.bpl();
                }

                /* BMI */
                0x30 => {
                    self.bmi();
                }

                /* BEQ */
                0xf0 => {
                    self.beq();
                }

                /* BCS */
                0xb0 => {
                    self.bcs();
                }

                /* BCC */
                0x90 => {
                    self.bcc();
                }

                /* BIT */
                0x24 | 0x2c => {
                    self.bit(&opcode.mode);
                }

                /* STA */
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    self.sta(&opcode.mode);
                }

                /* STX */
                0x86 | 0x96 | 0x8e => {
                    self.stx(&opcode.mode)
                }

                /* STY */
                0x84 | 0x94 | 0x8c => {
                    self.sty(&opcode.mode)
                }

                /* LDX */
                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => {
                    self.ldx(&opcode.mode);
                }

                /* LDY */
                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => {
                    self.ldy(&opcode.mode);
                }

                /* NOP */
                0xea => {
                    //do nothing
                }

                /* TAY */
                0xa8 => {
                    self.tay();
                }

                /* TSX */
                0xba => {
                    self.tsx();
                }

                /* TXA */
                0x8a => {
                    self.txa();
                }

                /* TXS */
                0x9a => {
                    self.txs();
                }

                /* TYA */
                0x98 => {
                    self.tya();
                }

                _ => todo!(),
            }
            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }

            callback(self);
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

    fn asl_accumulator(&mut self) {
        let mut data = self.register_a;
        if data >> 7 == 1 {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }
        data = data << 1;
        self.set_register_a(data)
    }

fn asl(&mut self, address_mode: &AddressingMode) -> u8 {
        let addr = self.calculate_address(address_mode);
        let mut data = self.bus.mem_read(addr);
        if data >> 7 == 1 {
            self.flags.insert(CpuFlags::CARRY);
        } else {
            self.flags.remove(CpuFlags::CARRY);
        }
        data = data << 1;
        self.bus.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }


    fn beq(&mut self) {
        let param = self.bus.mem_read(self.program_counter) as i8;
        if self.flags.contains(CpuFlags::ZERO) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bcc(&mut self) {
        let param = self.bus.mem_read(self.program_counter) as i8;
        if !self.flags.contains(CpuFlags::CARRY) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bcs(&mut self) {
        let param = self.bus.mem_read(self.program_counter) as i8;
        if self.flags.contains(CpuFlags::CARRY) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bit(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        let result = param & self.register_a;
        if result == 0 {
            self.flags.insert(CpuFlags::ZERO)
        } else {
            self.flags.remove(CpuFlags::ZERO)
        }
        if result & 0b0100_0000 == 0b0100_0000 {
            self.flags.insert(CpuFlags::OVERFLOW)
        }
        if result & 0b1000_0000 == 0b1000_0000 {
            self.flags.insert(CpuFlags::NEGATIVE)
        }
    }

    fn bmi(&mut self) {
        let param = self.bus.mem_read(self.program_counter) as i8;
        if self.flags.contains(CpuFlags::NEGATIVE) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bne(&mut self) {
        let param = self.bus.mem_read(self.program_counter) as i8;
        if !self.flags.contains(CpuFlags::ZERO) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bpl(&mut self) {
        let param = self.bus.mem_read(self.program_counter) as i8;
        if !self.flags.contains(CpuFlags::NEGATIVE) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bvc(&mut self) {
        let param = self.bus.mem_read(self.program_counter) as i8;
        if !self.flags.contains(CpuFlags::OVERFLOW) {
            self.program_counter = self.program_counter.wrapping_add(1).wrapping_add(param as u16)
        }
    }

    fn bvs(&mut self) {
        let param = self.bus.mem_read(self.program_counter) as i8;
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
        self.bus.mem_write(self.calculate_address(address_mode), result);
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
        let address = self.calculate_address(address_mode);
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

    fn lsr_accumulator(&mut self) {
        let m = self.register_a;
        if m & 1 == 1 {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.remove(CpuFlags::CARRY)
        }
        let data = m >> 1;
        self.set_register_a(data)
    }

    fn lsr(&mut self, address_mode: &AddressingMode) {
        let m = self.bus.mem_read(self.calculate_address(address_mode));
        if m & 1 == 1 {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.remove(CpuFlags::CARRY)
        }
        self.bus.mem_write(self.calculate_address(address_mode), m >> 1);
        let data = m >> 1;
        self.update_zero_and_negative_flags(data)
    }

    fn ora(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        let result = self.register_a | param;
        self.set_register_a(result)
    }

    fn pha(&mut self) {
        self.stack_push(self.register_a)
    }

    // http://wiki.nesdev.com/w/index.php/CPU_status_flag_behavior
    fn php(&mut self) {
        let mut flags = self.flags.clone();
        flags.insert(CpuFlags::BREAK);
        flags.insert(CpuFlags::BREAK2);
        self.stack_push(flags.bits());
    }

    fn pla(&mut self) {
        let data = self.stack_pop();
        self.set_register_a(data);
    }

    fn plp(&mut self) {
        self.flags = CpuFlags::from_bits_truncate(self.stack_pop());
        self.flags.remove(CpuFlags::BREAK);
        self.flags.insert(CpuFlags::BREAK2)
    }

    fn rol(&mut self, address_mode: &AddressingMode) {
        let param = self.bus.mem_read(self.calculate_address(address_mode));
        let old_carry = self.flags.contains(CpuFlags::CARRY);
        let old_bit_seven = (param >> 7) == 1;
        if old_bit_seven {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.remove(CpuFlags::CARRY)
        }
        let result = if old_carry {
            (param << 1) | 1
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

    fn rol_accumulator(&mut self) {
        let param = self.register_a;
        let old_carry = self.flags.contains(CpuFlags::CARRY);
        let old_bit_seven = (param >> 7) == 1;
        if old_bit_seven {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.remove(CpuFlags::CARRY)
        }
        let result = if old_carry {
            (param << 1) | 1
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
            (param >> 1) | 0b1000_0000
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
    fn ror_accumulator(&mut self) {
        let param = self.register_a;
        let old_carry = self.flags.contains(CpuFlags::CARRY);
        let old_bit_zero = param & 1;
        if old_bit_zero == 1 {
            self.flags.insert(CpuFlags::CARRY)
        } else {
            self.flags.remove(CpuFlags::CARRY)
        }
        let result = if old_carry {
            (param >> 1)| 0b1000_0000
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
        let addr = self.calculate_address(address_mode);
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

mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let bus = Bus::new();
        let mut cpu = Cpu::new(bus);
        cpu.load_and_run(&vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 5);
        assert!(cpu.flags.bits() & 0b0000_0010 == 0b00);
        assert!(cpu.flags.bits() & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let bus = Bus::new();
        let mut cpu = Cpu::new(bus);
        cpu.register_a = 10;
        cpu.load_and_run(&vec![0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let bus = Bus::new();
        let mut cpu = Cpu::new(bus);
        cpu.program_counter = 0x600;
        cpu.load_and_run(&vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let bus = Bus::new();
        let mut cpu = Cpu::new(bus);
        cpu.program_counter = 0x600;
        cpu.register_x = 0xff;
        cpu.load_and_run(&vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_lda_from_memory() {
        let bus = Bus::new();
        let mut cpu = Cpu::new(bus);
        cpu.bus.mem_write(0x10, 0x55);
        cpu.bus.mem_write(0xff, 0x65);
        cpu.program_counter = 0x600;
        cpu.load_and_run(&vec![0xa5, 0x10, 0xa5, 0xff,0x00]);

        assert_eq!(cpu.register_a, 0x65);
    }
}