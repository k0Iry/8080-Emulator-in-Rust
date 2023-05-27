use crate::{condition_codes::ConditionCodes, MemoryOutOfBounds, Result};

pub const RAM_SIZE: usize = 0x2000;
pub const ROM_SIZE: usize = 0x2000;

#[derive(Debug)]
#[repr(C)]
pub struct Cpu8080<'a> {
    pub ram: [u8; RAM_SIZE],
    pub rom: &'a [u8; ROM_SIZE],
    pub sp: u16,
    pub pc: u16,
    pub reg_a: u8,
    pub reg_b: u8,
    pub reg_c: u8,
    pub reg_d: u8,
    pub reg_e: u8,
    pub reg_h: u8,
    pub reg_l: u8,
    pub conditon_codes: ConditionCodes,
    pub interrupt_enabled: u8,
}

macro_rules! generate_move_from_mem {
    ( $( ($func:ident, $reg:ident) ),* ) => {
        $(
            fn $func(&mut self) -> Result<()> {
                let mem_addr = construct_address((self.reg_l, self.reg_h));
                Ok(self.$reg = self.load_byte_from_ram(mem_addr.into())?)
            }
        )*
    };
}

macro_rules! generate_store_reg_to_ram {
    ( $( ($func:ident, $reg:ident) ),* ) => {
        $(
            fn $func(&mut self) -> Result<()> {
                let mem_addr = construct_address((self.reg_l, self.reg_h));
                Ok(self.store_to_ram(mem_addr.into(), self.$reg)?)
            }
        )*
    };
}

macro_rules! generate_jump_on_condition {
    ( $( ($jump:ident, $condition:ident) ),* ) => {
        $(
            fn $jump(&mut self, $condition: bool) -> Result<()> {
                if $condition {
                    self.jmp()?;
                } else {
                    self.pc += 2;
                }
                Ok(())
            }
        )*
    };
}

macro_rules! generate_call_on_condition {
    ( $( ($call:ident, $condition:ident) ),* ) => {
        $(
            fn $call(&mut self, $condition: bool) -> Result<()> {
                if $condition {
                    self.call()?;
                } else {
                    self.pc += 2;
                }
                Ok(())
            }
        )*
    };
}

macro_rules! generate_return_on_condition {
    ( $( ($ret:ident, $condition:ident) ),* ) => {
        $(
            fn $ret(&mut self, $condition: bool) {
                if $condition {
                    self.ret();
                }
            }
        )*
    };
}

macro_rules! generate_load_data_into_reg_pair {
    ( $( ($func:ident, $reg_hi:ident, $reg_lo:ident) ),* ) => {
        $(
            fn $func(&mut self) -> Result<()> {
                (self.$reg_lo, self.$reg_hi) = self.load_d16_operand()?;
                self.pc += 2;
                Ok(())
            }
        )*
    };
}

macro_rules! generate_increment_reg_pair {
    ( $( ($func:ident, $reg_hi:ident, $reg_lo:ident) ),* ) => {
        $(
            fn $func(&mut self) {
                let big_endian_bytes = (construct_address((self.$reg_lo, self.$reg_hi)) + 1).to_be_bytes();
                self.$reg_hi = big_endian_bytes[0];
                self.$reg_lo = big_endian_bytes[1];
            }
        )*
    };
}

macro_rules! generate_decrement_reg_pair {
    ( $( ($func:ident, $reg_hi:ident, $reg_lo:ident) ),* ) => {
        $(
            fn $func(&mut self) {
                let big_endian_bytes = (construct_address((self.$reg_lo, self.$reg_hi)) - 1).to_be_bytes();
                self.$reg_hi = big_endian_bytes[0];
                self.$reg_lo = big_endian_bytes[1];
            }
        )*
    };
}

impl<'a> Cpu8080<'a> {
    fn add(&mut self, reg: u8) {
        let result = self.reg_a as u16 + reg as u16;
        self.reg_a = self.set_condition_bits(result, result > u8::MAX.into());
    }

    fn sub(&mut self, reg: u8) {
        let result = self.reg_a as u16 + reg.wrapping_neg() as u16;
        self.reg_a = self.set_condition_bits(result, result <= u8::MAX.into());
    }

    fn adc(&mut self, reg: u8) {
        let carry = if self.conditon_codes.is_carry_set() {
            1
        } else {
            0
        };
        let result = self.reg_a as u16 + reg as u16 + carry;
        self.reg_a = self.set_condition_bits(result, result > u8::MAX.into());
    }

    fn sbb(&mut self, reg: u8) {
        let carry: u8 = if self.conditon_codes.is_carry_set() {
            1
        } else {
            0
        };
        let result = self.reg_a as u16 + reg.wrapping_neg() as u16 + carry.wrapping_neg() as u16;
        self.reg_a = self.set_condition_bits(result, result <= u8::MAX.into());
    }

    fn set_condition_bits(&mut self, result: u16, is_carry: bool) -> u8 {
        let lsb = result as u8;
        if lsb == 0 {
            self.conditon_codes.set_zero();
        } else {
            self.conditon_codes.reset_zero();
        }
        if lsb >= 0x80 {
            self.conditon_codes.set_sign();
        } else {
            self.conditon_codes.reset_sign();
        }
        if is_carry {
            self.conditon_codes.set_carry();
        } else {
            self.conditon_codes.reset_carry();
        }
        if lsb.count_ones() % 2 == 0 {
            self.conditon_codes.set_parity();
        } else {
            self.conditon_codes.reset_parity();
        }
        lsb
    }

    fn load_byte_from_ram(&self, addr: usize) -> Result<u8> {
        Ok(*self
            .ram
            .get((addr - ROM_SIZE) as usize)
            .ok_or(MemoryOutOfBounds)?)
    }

    fn store_to_ram(&mut self, addr: usize, value: u8) -> Result<()> {
        Ok(*self.ram.get_mut(addr - ROM_SIZE).ok_or(MemoryOutOfBounds)? = value)
    }

    fn adi(&mut self) -> Result<()> {
        let imm = self.load_d8_operand()?;
        self.pc += 1;
        Ok(self.add(imm))
    }

    fn aci(&mut self) -> Result<()> {
        let imm = self.load_d8_operand()?;
        self.pc += 1;
        Ok(self.adc(imm))
    }

    fn add_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_ram(mem_addr.into())?;
        Ok(self.add(value))
    }

    fn sub_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_ram(mem_addr.into())?;
        Ok(self.sub(value))
    }

    fn adc_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_ram(mem_addr.into())?;
        Ok(self.adc(value))
    }

    fn sbb_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_ram(mem_addr.into())?;
        Ok(self.sbb(value))
    }

    fn sui(&mut self) -> Result<()> {
        let imm = self.load_d8_operand()?;
        self.pc += 1;
        Ok(self.sub(imm))
    }

    fn sbi(&mut self) -> Result<()> {
        let imm = self.load_d8_operand()?;
        self.pc += 1;
        Ok(self.sbb(imm))
    }

    generate_increment_reg_pair![
        (inx_b, reg_b, reg_c),
        (inx_d, reg_d, reg_e),
        (inx_h, reg_h, reg_l)
    ];

    generate_decrement_reg_pair![
        (dcx_b, reg_b, reg_c),
        (dcx_d, reg_d, reg_e),
        (dcx_h, reg_h, reg_l)
    ];

    generate_move_from_mem![
        (move_from_mem_to_b, reg_b),
        (move_from_mem_to_c, reg_c),
        (move_from_mem_to_d, reg_d),
        (move_from_mem_to_e, reg_e),
        (move_from_mem_to_l, reg_l),
        (move_from_mem_to_h, reg_h),
        (move_from_mem_to_a, reg_a)
    ];

    generate_store_reg_to_ram![
        (store_reg_b_to_ram, reg_b),
        (store_reg_c_to_ram, reg_c),
        (store_reg_d_to_ram, reg_d),
        (store_reg_e_to_ram, reg_e),
        (store_reg_l_to_ram, reg_l),
        (store_reg_h_to_ram, reg_h),
        (store_reg_a_to_ram, reg_a)
    ];

    generate_load_data_into_reg_pair![
        (load_data_into_reg_pair_b, reg_b, reg_c),
        (load_data_into_reg_pair_d, reg_d, reg_e),
        (load_data_into_reg_pair_h, reg_h, reg_l)
    ];

    pub fn execute(&mut self) -> Result<()> {
        let opcode = self.rom.get(self.pc as usize).ok_or(MemoryOutOfBounds)?;
        match *opcode {
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0x40 | 0x49 | 0x52 | 0x5b
            | 0x64 | 0x6d | 0x7f | 0xcb | 0xd9 | 0xdd | 0xed | 0xfd => (),
            0x01 => self.load_data_into_reg_pair_b()?,
            0x02 => self.store_to_ram(
                construct_address((self.reg_c, self.reg_b)).into(),
                self.reg_a,
            )?,
            0x03 => self.inx_b(),
            0x0b => self.dcx_b(),
            0x11 => self.load_data_into_reg_pair_d()?,
            0x12 => self.store_to_ram(
                construct_address((self.reg_e, self.reg_d)).into(),
                self.reg_a,
            )?,
            0x13 => self.inx_d(),
            0x1b => self.dcx_d(),
            0x21 => self.load_data_into_reg_pair_h()?,
            0x23 => self.inx_h(),
            0x2b => self.dcx_h(),
            0x31 => self.load_stack_pointer_from_operand()?,
            0x33 => self.sp += 1,
            0x3b => self.sp -= 1,
            0x41 => self.reg_b = self.reg_c,
            0x42 => self.reg_b = self.reg_d,
            0x43 => self.reg_b = self.reg_e,
            0x44 => self.reg_b = self.reg_h,
            0x45 => self.reg_b = self.reg_l,
            0x46 => self.move_from_mem_to_b()?,
            0x47 => self.reg_b = self.reg_a,
            0x48 => self.reg_c = self.reg_b,
            0x4a => self.reg_c = self.reg_d,
            0x4b => self.reg_c = self.reg_e,
            0x4c => self.reg_c = self.reg_h,
            0x4d => self.reg_c = self.reg_l,
            0x4e => self.move_from_mem_to_c()?,
            0x4f => self.reg_c = self.reg_a,
            0x50 => self.reg_d = self.reg_b,
            0x51 => self.reg_d = self.reg_c,
            0x53 => self.reg_d = self.reg_e,
            0x54 => self.reg_d = self.reg_h,
            0x55 => self.reg_d = self.reg_l,
            0x56 => self.move_from_mem_to_d()?,
            0x57 => self.reg_d = self.reg_a,
            0x58 => self.reg_e = self.reg_b,
            0x59 => self.reg_e = self.reg_c,
            0x5a => self.reg_e = self.reg_d,
            0x5c => self.reg_e = self.reg_h,
            0x5d => self.reg_e = self.reg_l,
            0x5e => self.move_from_mem_to_e()?,
            0x5f => self.reg_e = self.reg_a,
            0x60 => self.reg_h = self.reg_b,
            0x61 => self.reg_h = self.reg_c,
            0x62 => self.reg_h = self.reg_d,
            0x63 => self.reg_h = self.reg_e,
            0x65 => self.reg_h = self.reg_l,
            0x66 => self.move_from_mem_to_h()?,
            0x67 => self.reg_h = self.reg_a,
            0x68 => self.reg_l = self.reg_b,
            0x69 => self.reg_l = self.reg_c,
            0x6a => self.reg_l = self.reg_d,
            0x6b => self.reg_l = self.reg_e,
            0x6c => self.reg_l = self.reg_h,
            0x6e => self.move_from_mem_to_l()?,
            0x6f => self.reg_l = self.reg_a,
            0x70 => self.store_reg_b_to_ram()?,
            0x71 => self.store_reg_c_to_ram()?,
            0x72 => self.store_reg_d_to_ram()?,
            0x73 => self.store_reg_e_to_ram()?,
            0x74 => self.store_reg_h_to_ram()?,
            0x75 => self.store_reg_l_to_ram()?,
            // 0x76 =>
            0x77 => self.store_reg_a_to_ram()?,
            0x78 => self.reg_a = self.reg_b,
            0x79 => self.reg_a = self.reg_c,
            0x7a => self.reg_a = self.reg_d,
            0x7b => self.reg_a = self.reg_e,
            0x7c => self.reg_a = self.reg_h,
            0x7d => self.reg_a = self.reg_l,
            0x7e => self.move_from_mem_to_a()?,
            0x80 => self.add(self.reg_b),
            0x81 => self.add(self.reg_c),
            0x82 => self.add(self.reg_d),
            0x83 => self.add(self.reg_e),
            0x84 => self.add(self.reg_h),
            0x85 => self.add(self.reg_l),
            0x86 => self.add_m()?,
            0x87 => self.add(self.reg_a),
            0x88 => self.adc(self.reg_b),
            0x89 => self.adc(self.reg_c),
            0x8a => self.adc(self.reg_d),
            0x8b => self.adc(self.reg_e),
            0x8c => self.adc(self.reg_h),
            0x8d => self.adc(self.reg_l),
            0x8e => self.adc_m()?,
            0x8f => self.adc(self.reg_a),
            0x90 => self.sub(self.reg_b),
            0x91 => self.sub(self.reg_c),
            0x92 => self.sub(self.reg_d),
            0x93 => self.sub(self.reg_e),
            0x94 => self.sub(self.reg_h),
            0x95 => self.sub(self.reg_l),
            0x96 => self.sub_m()?,
            0x97 => self.sub(self.reg_a),
            0x98 => self.sbb(self.reg_b),
            0x99 => self.sbb(self.reg_c),
            0x9a => self.sbb(self.reg_d),
            0x9b => self.sbb(self.reg_e),
            0x9c => self.sbb(self.reg_h),
            0x9d => self.sbb(self.reg_l),
            0x9e => self.sbb_m()?,
            0x9f => self.sbb(self.reg_a),
            0xc0 => self.ret_on_zero(!self.conditon_codes.is_zero_set()),
            0xc2 => self.jump_on_zero(!self.conditon_codes.is_zero_set())?,
            0xc3 => self.jmp()?,
            0xc8 => self.ret_on_zero(self.conditon_codes.is_zero_set()),
            0xca => self.jump_on_zero(self.conditon_codes.is_zero_set())?,
            0xc4 => self.call_on_zero(!self.conditon_codes.is_zero_set())?,
            0xc6 => self.adi()?,
            0xc9 => self.ret(),
            0xcc => self.call_on_zero(self.conditon_codes.is_zero_set())?,
            0xcd => self.call()?,
            0xce => self.aci()?,
            0xd0 => self.ret_on_carry(!self.conditon_codes.is_carry_set()),
            0xd2 => self.jump_on_carry(!self.conditon_codes.is_carry_set())?,
            0xd4 => self.call_on_carry(!self.conditon_codes.is_carry_set())?,
            0xd6 => self.sui()?,
            0xd8 => self.ret_on_carry(self.conditon_codes.is_carry_set()),
            0xda => self.jump_on_carry(self.conditon_codes.is_carry_set())?,
            0xdc => self.call_on_carry(self.conditon_codes.is_carry_set())?,
            0xde => self.sbi()?,
            0xe0 => self.ret_on_parity(!self.conditon_codes.is_parity()),
            0xe2 => self.jump_on_parity(!self.conditon_codes.is_parity())?,
            0xe4 => self.call_on_parity(!self.conditon_codes.is_parity())?,
            0xe8 => self.ret_on_parity(self.conditon_codes.is_parity()),
            0xea => self.jump_on_parity(self.conditon_codes.is_parity())?,
            0xec => self.call_on_parity(self.conditon_codes.is_parity())?,
            0xf0 => self.ret_on_sign(!self.conditon_codes.is_sign()),
            0xf2 => self.jump_on_sign(!self.conditon_codes.is_sign())?,
            0xf4 => self.call_on_sign(!self.conditon_codes.is_sign())?,
            0xf8 => self.ret_on_sign(self.conditon_codes.is_sign()),
            0xfa => self.jump_on_sign(self.conditon_codes.is_sign())?,
            0xfc => self.call_on_sign(self.conditon_codes.is_sign())?,
            _ => (),
        }
        self.pc += 1;
        Ok(())
    }

    fn load_stack_pointer_from_operand(&mut self) -> Result<()> {
        self.sp = construct_address(self.load_d16_operand()?);
        self.pc += 2;
        Ok(())
    }

    generate_call_on_condition![
        (call_on_zero, is_zero_set),
        (call_on_carry, is_carry_set),
        (call_on_parity, is_parity_set),
        (call_on_sign, is_sign_set)
    ];

    fn call(&mut self) -> Result<()> {
        let pc_in_bytes = (self.pc + 2).to_be_bytes();
        (
            self.ram[self.sp as usize - 1 - ROM_SIZE],
            self.ram[self.sp as usize - 2 - ROM_SIZE],
        ) = (pc_in_bytes[0], pc_in_bytes[1]);
        self.sp -= 2;
        Ok(self.pc = construct_address(self.load_d16_operand()?))
    }

    generate_return_on_condition![
        (ret_on_zero, is_zero_set),
        (ret_on_carry, is_carry_set),
        (ret_on_parity, is_parity_set),
        (ret_on_sign, is_sign_set)
    ];

    fn ret(&mut self) {
        let addr_lo = self.ram[self.sp as usize - ROM_SIZE];
        let addr_hi = self.ram[self.sp as usize + 1 - ROM_SIZE];
        self.pc = construct_address((addr_lo, addr_hi));
        self.sp += 2;
    }

    /// get operand parts in (lo, hi)
    fn load_d16_operand(&self) -> Result<(u8, u8)> {
        Ok((
            *self
                .rom
                .get((self.pc + 1) as usize)
                .ok_or(MemoryOutOfBounds)?,
            *self
                .rom
                .get((self.pc + 2) as usize)
                .ok_or(MemoryOutOfBounds)?,
        ))
    }

    fn load_d8_operand(&self) -> Result<u8> {
        Ok(*self
            .rom
            .get((self.pc + 1) as usize)
            .ok_or(MemoryOutOfBounds)?)
    }

    generate_jump_on_condition![
        (jump_on_zero, is_zero_set),
        (jump_on_carry, is_carry_set),
        (jump_on_parity, is_parity_set),
        (jump_on_sign, is_sign_set)
    ];

    fn jmp(&mut self) -> Result<()> {
        Ok(self.pc = construct_address(self.load_d16_operand()?) - 1)
    }
}

#[inline(always)]
fn construct_address((low_addr, high_addr): (u8, u8)) -> u16 {
    (high_addr as u16) << 8 | (low_addr as u16)
}