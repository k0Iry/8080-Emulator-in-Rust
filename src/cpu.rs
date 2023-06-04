use crate::{condition_codes::ConditionCodes, MemoryOutOfBounds, Result};

const RAM_SIZE: usize = 0x2000;
pub const ROM_SIZE: usize = 0x2000;

#[derive(Debug)]
#[repr(C)]
pub struct Cpu8080<'a> {
    ram: [u8; RAM_SIZE],
    rom: &'a [u8; ROM_SIZE],
    sp: u16,
    pc: u16,
    reg_a: u8,
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_h: u8,
    reg_l: u8,
    conditon_codes: ConditionCodes,
    interrupt_enabled: u8,
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
                self.store_to_ram(mem_addr.into(), self.$reg)
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

macro_rules! generate_inc_dec_reg_pair {
    ( $( ($func:ident, $reg_hi:ident, $reg_lo:ident, $value:expr) ),* ) => {
        $(
            fn $func(&mut self) {
                let big_endian_bytes = (construct_address((self.$reg_lo, self.$reg_hi)) + $value).to_be_bytes();
                self.$reg_hi = big_endian_bytes[0];
                self.$reg_lo = big_endian_bytes[1];
            }
        )*
    };
}

macro_rules! generate_inc_dec_reg {
    ( $( ($func:ident, $reg:ident, $value:expr) ),* ) => {
        $(
            fn $func(&mut self) {
                self.$reg = self.set_condition_bits(self.$reg.into(), $value) as u8;
            }
        )*
    };
}

impl<'a> Cpu8080<'a> {
    pub fn new(rom: &'a [u8; ROM_SIZE]) -> Self {
        Cpu8080 {
            reg_a: 0,
            reg_b: 0,
            reg_c: 0,
            reg_d: 0,
            reg_e: 0,
            reg_h: 0,
            reg_l: 0,
            sp: 0,
            pc: 0,
            rom,
            ram: [0; RAM_SIZE],
            conditon_codes: ConditionCodes::default(),
            interrupt_enabled: 0,
        }
    }

    fn add(&mut self, reg: u8) {
        let result = self.set_condition_bits(self.reg_a.into(), reg.into());
        self.set_carry(result > u8::MAX.into());
        self.reg_a = result as u8;
    }

    fn sub(&mut self, reg: u8) {
        let result = self.set_condition_bits(self.reg_a.into(), reg.wrapping_neg().into());
        self.set_carry(result <= u8::MAX.into());
        self.reg_a = result as u8;
    }

    fn adc(&mut self, reg: u8) {
        let carry = if self.conditon_codes.is_carry_set() {
            1
        } else {
            0
        };
        let result = self.set_condition_bits(self.reg_a.into(), reg as u16 + carry);
        self.set_carry(result > u8::MAX.into());
        self.reg_a = result as u8;
    }

    fn sbb(&mut self, reg: u8) {
        let carry: u8 = if self.conditon_codes.is_carry_set() {
            1
        } else {
            0
        };
        let result = self.set_condition_bits(
            self.reg_a.into(),
            reg.wrapping_neg() as u16 + carry.wrapping_neg() as u16,
        );
        self.set_carry(result <= u8::MAX.into());
        self.reg_a = result as u8;
    }

    fn set_condition_bits(&mut self, value1: u16, value2: u16) -> u16 {
        // Z, S, P, AC
        let result = value1 + value2;
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
        if lsb.count_ones() % 2 == 0 {
            self.conditon_codes.set_parity();
        } else {
            self.conditon_codes.reset_parity();
        }
        let aux_carry = result & 0xf;
        if aux_carry < (value1 & 0xf) && aux_carry < (value2 & 0xf) {
            self.conditon_codes.set_aux_carry()
        } else {
            self.conditon_codes.reset_aux_carry()
        }
        result
    }

    fn set_carry(&mut self, is_carry: bool) {
        if is_carry {
            self.conditon_codes.set_carry();
        } else {
            self.conditon_codes.reset_carry();
        }
    }

    fn dad(&mut self, value: u16) {
        let hl = construct_address((self.reg_l, self.reg_h)) as u32;
        let hl = hl + value as u32;
        self.set_carry(hl > u16::MAX.into());
    }

    fn load_byte_from_ram(&self, addr: usize) -> Result<u8> {
        Ok(*self.ram.get(addr - ROM_SIZE).ok_or(MemoryOutOfBounds)?)
    }

    fn store_to_ram(&mut self, addr: usize, value: u8) -> Result<()> {
        *self.ram.get_mut(addr - ROM_SIZE).ok_or(MemoryOutOfBounds)? = value;
        Ok(())
    }

    fn adi(&mut self) -> Result<()> {
        let imm = self.load_d8_operand()?;
        self.add(imm);
        Ok(())
    }

    fn aci(&mut self) -> Result<()> {
        let imm = self.load_d8_operand()?;
        self.adc(imm);
        Ok(())
    }

    fn add_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_ram(mem_addr.into())?;
        self.add(value);
        Ok(())
    }

    fn sub_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_ram(mem_addr.into())?;
        self.sub(value);
        Ok(())
    }

    fn adc_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_ram(mem_addr.into())?;
        self.adc(value);
        Ok(())
    }

    fn sbb_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_ram(mem_addr.into())?;
        self.sbb(value);
        Ok(())
    }

    fn sui(&mut self) -> Result<()> {
        let imm = self.load_d8_operand()?;
        self.sub(imm);
        Ok(())
    }

    fn sbi(&mut self) -> Result<()> {
        let imm = self.load_d8_operand()?;
        self.sbb(imm);
        Ok(())
    }

    generate_inc_dec_reg_pair![
        (inx_b, reg_b, reg_c, 1),
        (inx_d, reg_d, reg_e, 1),
        (inx_h, reg_h, reg_l, 1),
        (dcx_b, reg_b, reg_c, 1u16.wrapping_neg()),
        (dcx_d, reg_d, reg_e, 1u16.wrapping_neg()),
        (dcx_h, reg_h, reg_l, 1u16.wrapping_neg())
    ];

    generate_inc_dec_reg![
        (inr_b, reg_b, 1),
        (inr_c, reg_c, 1),
        (inr_d, reg_d, 1),
        (inr_e, reg_e, 1),
        (inr_h, reg_h, 1),
        (inr_l, reg_l, 1),
        (inr_a, reg_a, 1),
        (dcr_b, reg_b, 1u16.wrapping_neg()),
        (dcr_c, reg_c, 1u16.wrapping_neg()),
        (dcr_d, reg_d, 1u16.wrapping_neg()),
        (dcr_e, reg_e, 1u16.wrapping_neg()),
        (dcr_h, reg_h, 1u16.wrapping_neg()),
        (dcr_l, reg_l, 1u16.wrapping_neg()),
        (drc_a, reg_a, 1u16.wrapping_neg())
    ];

    fn inr_m(&mut self) -> Result<()> {
        let addr: usize = construct_address((self.reg_l, self.reg_h)).into();
        self.store_to_ram(addr, self.load_byte_from_ram(addr)? + 1)?;
        Ok(())
    }

    fn dcr_m(&mut self) -> Result<()> {
        let addr: usize = construct_address((self.reg_l, self.reg_h)).into();
        self.store_to_ram(addr, self.load_byte_from_ram(addr)? - 1)?;
        Ok(())
    }

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

    pub fn run(&mut self) -> Result<()> {
        while self.pc < ROM_SIZE as u16 {
            self.execute()?
        }
        Ok(())
    }

    fn execute(&mut self) -> Result<()> {
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
            0x04 => self.inr_b(),
            0x05 => self.dcr_b(),
            0x06 => self.reg_b = self.load_d8_operand()?,
            0x09 => self.dad(construct_address((self.reg_c, self.reg_b))),
            0x0b => self.dcx_b(),
            0x0c => self.inr_c(),
            0x0d => self.dcr_c(),
            0x0e => self.reg_c = self.load_d8_operand()?,
            0x11 => self.load_data_into_reg_pair_d()?,
            0x12 => self.store_to_ram(
                construct_address((self.reg_e, self.reg_d)).into(),
                self.reg_a,
            )?,
            0x13 => self.inx_d(),
            0x14 => self.inr_d(),
            0x15 => self.dcr_d(),
            0x16 => self.reg_d = self.load_d8_operand()?,
            0x19 => self.dad(construct_address((self.reg_e, self.reg_d))),
            0x1b => self.dcx_d(),
            0x1c => self.inr_e(),
            0x1d => self.dcr_e(),
            0x1e => self.reg_e = self.load_d8_operand()?,
            0x21 => self.load_data_into_reg_pair_h()?,
            0x23 => self.inx_h(),
            0x24 => self.inr_h(),
            0x25 => self.dcr_h(),
            0x26 => self.reg_h = self.load_d8_operand()?,
            0x29 => self.dad(construct_address((self.reg_l, self.reg_h))),
            0x2b => self.dcx_h(),
            0x2c => self.inr_l(),
            0x2d => self.dcr_l(),
            0x2e => self.reg_l = self.load_d8_operand()?,
            0x31 => self.load_stack_pointer_from_operand()?,
            0x33 => self.sp += 1,
            0x34 => self.inr_m()?,
            0x35 => self.dcr_m()?,
            0x36 => {
                let imm = self.load_d8_operand()?;
                self.store_to_ram(construct_address((self.reg_l, self.reg_h)).into(), imm)?
            }
            0x39 => self.dad(self.sp),
            0x3b => self.sp -= 1,
            0x3c => self.inr_a(),
            0x3d => self.drc_a(),
            0x3e => self.reg_a = self.load_d8_operand()?,
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
        self.pc = construct_address(self.load_d16_operand()?) - 1;
        Ok(())
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

    fn load_d8_operand(&mut self) -> Result<u8> {
        let value = *self
            .rom
            .get((self.pc + 1) as usize)
            .ok_or(MemoryOutOfBounds)?;
        self.pc += 1;
        Ok(value)
    }

    generate_jump_on_condition![
        (jump_on_zero, is_zero_set),
        (jump_on_carry, is_carry_set),
        (jump_on_parity, is_parity_set),
        (jump_on_sign, is_sign_set)
    ];

    fn jmp(&mut self) -> Result<()> {
        self.pc = construct_address(self.load_d16_operand()?) - 1;
        Ok(())
    }
}

#[inline(always)]
fn construct_address((low_addr, high_addr): (u8, u8)) -> u16 {
    (high_addr as u16) << 8 | (low_addr as u16)
}
