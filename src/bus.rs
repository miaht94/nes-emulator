pub trait Memory {
    fn mem_read(&self, address: u16) -> u8;
    fn mem_write(&mut self, address: u16, value: u8);
    fn mem_read_u16(&self, address: u16) -> u16;
    fn mem_write_u16(&mut self, address: u16, value: u16);
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

pub struct Bus {
    cpu_vram: [u8; 0x800]
}
impl Bus { 
    pub fn new() -> Self {
        Bus { cpu_vram: [0; 0x800] }
    }

    fn get_real_address(&self, address: u16) -> usize { 
        let address = match address {
            RAM ..= RAM_MIRRORS_END => address & 0b111_1111_1111,
            PPU_REGISTERS ..= PPU_REGISTERS_MIRRORS_END => {
                address & 0b0010_0000_0000_0111;
                todo!("Not implemented")
            },
            _ => panic!("Ignoring memory access at {}", address)
        };
        address as usize
    } 
}
impl Memory for Bus {
    fn mem_read(&self, address: u16) -> u8 {
        let address = self.get_real_address(address);
        self.cpu_vram[address]
    }

    fn mem_write(&mut self, address: u16, value: u8) {
        let address: usize = self.get_real_address(address);
        self.cpu_vram[address] = value
    }
    
    fn mem_read_u16(&self, address: u16) -> u16 {
        let low = self.mem_read(address) as u16;
        let high = self.mem_read(address + 1) as u16;
        high << 8 | low
    }
    
    fn mem_write_u16(&mut self, address: u16, value: u16) {
        let low = value as u8;
        let high = (value >> 8) as u8;
        self.mem_write(address, low);
        self.mem_write(address + 1, high);
    }    
}