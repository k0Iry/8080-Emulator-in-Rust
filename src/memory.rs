pub trait Memory {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, value: u8) -> bool;
    fn reset_ram(&mut self);
}

pub struct SplitMemory<'a> {
    rom: &'a [u8],
    ram: &'a mut [u8],
}

impl<'a> SplitMemory<'a> {
    pub fn new(rom: &'a [u8], ram: &'a mut [u8]) -> Self {
        Self { rom, ram }
    }

    pub fn rom(&self) -> &[u8] {
        self.rom
    }

    pub fn ram(&self) -> &[u8] {
        self.ram
    }

    pub fn ram_mut(&mut self) -> &mut [u8] {
        self.ram
    }
}

impl Memory for SplitMemory<'_> {
    fn read(&self, addr: u16) -> Option<u8> {
        let addr = addr as usize;
        if addr < self.rom.len() {
            self.rom.get(addr).copied()
        } else {
            self.ram.get(addr - self.rom.len()).copied()
        }
    }

    fn write(&mut self, addr: u16, value: u8) -> bool {
        let addr = addr as usize;
        if addr < self.rom.len() {
            return false;
        }
        if let Some(cell) = self.ram.get_mut(addr - self.rom.len()) {
            *cell = value;
            true
        } else {
            false
        }
    }

    fn reset_ram(&mut self) {
        self.ram.fill(0);
    }
}
