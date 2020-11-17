use crate::register::BitTwiddle;

pub trait ReadArray {
    fn read_byte(&self, address: u32) -> u32;
    fn read_half(&self, address: u32) -> u32;
    fn read_word(&self, address: u32) -> u32;
}

pub trait WriteArray {
    fn write_byte(&mut self, address: u32, value: u32);
    fn write_half(&mut self, address: u32, value: u32);
    fn write_word(&mut self, address: u32, value: u32);
}

impl ReadArray for &[u8] {
    fn read_byte(&self, address: u32) -> u32 {
        let address = address as usize;
        self[address] as u32
    }

    fn read_half(&self, address: u32) -> u32 {
        let address = address as usize;
        self[address] as u32 | (self[address + 1] as u32) << 8
    }

    fn read_word(&self, address: u32) -> u32 {
        let address = address as usize;
        self[address] as u32 |
            (self[address + 1] as u32) << 8 |
            (self[address + 2] as u32) << 16 |
            (self[address + 3] as u32) << 24
    }
}

impl WriteArray for &mut [u8] {
    fn write_byte(&mut self, address: u32, value: u32) {
        let address = address as usize;
        self[address] = value.byte() as u8;
    }

    fn write_half(&mut self, address: u32, value: u32) {
        let address = address as usize;
        self[address] = value.byte() as u8;
        self[address + 1] = value.upper_bits(24).byte() as u8;
    }

    fn write_word(&mut self, address: u32, value: u32) {
        let address = address as usize;
        self[address] = value.byte() as u8;
        self[address + 1] = value.upper_bits(24).byte() as u8;
        self[address + 2] = value.upper_bits(16).byte() as u8;
        self[address + 3] = value.upper_bits(8).byte() as u8;
    }
}

pub fn get_rs(op: u32) -> u32 {
    op.range(21, 25)
}
pub fn get_rt(op: u32) -> u32 {
    op.range(16, 20)
}
pub fn get_rd(op: u32) -> u32 {
    op.range(11, 15)
}
pub fn get_imm5(op: u32) -> u32 {
    op.range(6, 10)
}
pub fn get_imm16(op: u32) -> u32 {
    op.half()
}
pub fn get_imm25(op: u32) -> u32 {
    op.lowest_bits(25)
}
pub fn get_imm26(op: u32) -> u32 {
    op.lowest_bits(26)
}
pub fn get_primary_field(op: u32) -> u32 {
    op.upper_bits(6)
}
pub fn get_secondary_field(op: u32) -> u32 {
    op.lowest_bits(6)
}
