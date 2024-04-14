enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
}

struct Cpu {
    program_counter: u16,
    register_a: u8,
    register_x: u8,
    register_y: u8,
    stack_pointer: u8,
    memory: [u8; 0xffff]
}

impl Cpu {
    fn new() -> Self {
        Cpu { program_counter: 0, register_a: 0, register_x: 0, register_y: 0, stack_pointer: 0, memory: [0 as u8; 0xff as usize] }
    }

    fn calculate_address(&self, address_mode: &AddressingMode) -> u16 {
        match address_mode {
            &AddressingMode::Immediate => 1,
            _ => 1
        }
    }

    fn load(&mut self, program: &Vec<u8>) {
        self.memory[0x8000 ..].copy_from_slice(&program);
        self.program_counter = 0x8000;
    }

    fn load_and_run(&mut self, program: &Vec<u8>) {
        self.load(program);
        self.run();
    }

    fn run(&mut self) {
        self.program_counter = 0;
        loop {
            let ops_code = program[self.program_counter as usize];
            self.program_counter += 1;
            match ops_code {
                0x00 => todo!(),
                0xA9 => todo!(),
                _ => todo!()
            }
        }
    }
}