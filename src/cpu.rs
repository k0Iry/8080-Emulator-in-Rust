use core::{panic, time};
use std::{
    mem,
    ops::{Deref, DerefMut},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use std::sync::mpsc::channel;

use crate::{condition_codes::ConditionCodes, MemoryOutOfBounds, Result, CLOCK_CYCLES};

const RAM_SIZE: usize = 0x2000;

#[derive(Debug)]
#[repr(C)]
pub struct Cpu8080<'a> {
    ram: [u8; RAM_SIZE],
    rom: &'a Vec<u8>,
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
    interrupt_enabled: bool,
}

macro_rules! generate_move_from_mem {
    ( $( ($func:ident, $reg:ident) ),* ) => {
        $(
            fn $func(&mut self) -> Result<()> {
                let mem_addr = construct_address((self.reg_l, self.reg_h));
                Ok(self.$reg = self.load_byte_from_memory(mem_addr.into())?)
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
            fn $ret(&mut self, $condition: bool) -> Result<()> {
                if $condition {
                    self.ret()?;
                }
                Ok(())
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
                let pair_value = construct_address((self.$reg_lo, self.$reg_hi)) as u32;
                let value = $value as u32;
                let big_endian_bytes = (pair_value + value).to_be_bytes();
                self.$reg_hi = big_endian_bytes[2];
                self.$reg_lo = big_endian_bytes[3];
            }
        )*
    };
}

macro_rules! generate_inc_dec_reg {
    ( $( ($func:ident, $reg:ident, $value:expr) ),* ) => {
        $(
            fn $func(&mut self) {
                self.$reg = self.set_condition_bits(self.$reg.into(), $value.into()) as u8;
            }
        )*
    };
}

macro_rules! pop_to_reg_pair {
    ( $( ($func:ident, $reg_hi:ident, $reg_lo:ident) ),* ) => {
        $(
            fn $func(&mut self) -> Result<()> {
                let addr_lo = self.load_byte_from_memory(self.sp.into())?;
                let addr_hi = self.load_byte_from_memory((self.sp + 1).into())?;
                (self.$reg_lo, self.$reg_hi) = (addr_lo, addr_hi);
                self.sp += 2;
                Ok(())
            }
        )*
    };
}

macro_rules! push_to_reg_pair {
    ( $( ($func:ident, $reg_hi:ident, $reg_lo:ident) ),* ) => {
        $(
            fn $func(&mut self) -> Result<()> {
                self.store_to_ram((self.sp - 1).into(), self.$reg_hi)?;
                self.store_to_ram((self.sp - 2).into(), self.$reg_lo)?;
                self.sp -= 2;
                Ok(())
            }
        )*
    };
}

impl<'a> Cpu8080<'a> {
    pub fn new(rom: &'a Vec<u8>) -> Self {
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
            interrupt_enabled: false,
        }
    }

    fn add(&mut self, reg: u8) {
        let result = self.set_condition_bits(self.reg_a.into(), reg.into());
        self.set_carry(result > u8::MAX.into());
        self.reg_a = result as u8;
    }

    fn sub(&mut self, reg: u8) {
        let result = self.set_condition_bits(self.reg_a.into(), reg.wrapping_neg().into());
        self.set_carry(self.reg_a < reg);
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
        self.set_carry(self.reg_a < reg);
        self.reg_a = result as u8;
    }

    fn set_condition_bits(&mut self, value1: u16, value2: u16) -> u16 {
        // Z, S, P, AC
        let result = value1 + value2;
        let lsb = result as u8;
        self.set_zero(lsb == 0);
        self.set_sign(lsb >= 0x80);
        self.set_parity(lsb.count_ones() % 2 == 0);
        let aux_carry = result & 0xf;
        if aux_carry < (value1 & 0xf) && aux_carry < (value2 & 0xf) {
            self.conditon_codes.set_aux_carry()
        } else {
            self.conditon_codes.reset_aux_carry()
        }
        result
    }

    fn set_zero(&mut self, is_zero: bool) {
        if is_zero {
            self.conditon_codes.set_zero()
        } else {
            self.conditon_codes.reset_zero()
        }
    }

    fn set_parity(&mut self, is_parity: bool) {
        if is_parity {
            self.conditon_codes.set_parity()
        } else {
            self.conditon_codes.reset_parity()
        }
    }

    fn set_sign(&mut self, is_sign: bool) {
        if is_sign {
            self.conditon_codes.set_sign()
        } else {
            self.conditon_codes.reset_sign()
        }
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
        let be_bytes = (hl as u16).to_be_bytes();
        (self.reg_l, self.reg_h) = (be_bytes[1], be_bytes[0]);
        self.set_carry(hl > u16::MAX.into());
    }

    fn rlc(&mut self) {
        self.reg_a = self.reg_a.rotate_left(1);
        let carry = self.reg_a & 0x1;
        self.set_carry(carry == 1);
    }

    fn rrc(&mut self) {
        let carry = self.reg_a & 0x1;
        self.reg_a = self.reg_a.rotate_right(1);
        self.set_carry(carry == 1);
    }

    fn ral(&mut self) {
        let carry = self.reg_a >= 0x80;
        self.reg_a <<= 1;
        self.reg_a |= self.conditon_codes.is_carry_set() as u8;
        self.set_carry(carry);
    }

    fn rar(&mut self) {
        let carry = self.reg_a & 0x1;
        self.reg_a >>= 1;
        self.reg_a |= (self.conditon_codes.is_carry_set() as u8) << 7;
        self.set_carry(carry == 1);
    }

    /// It is allowed to load content from either ROM or RAM
    fn load_byte_from_memory(&self, addr: usize) -> Result<u8> {
        if addr >= self.rom.len() {
            Ok(*self
                .ram
                .get(addr - self.rom.len())
                .ok_or(MemoryOutOfBounds)?)
        } else {
            Ok(*self.rom.get(addr).ok_or(MemoryOutOfBounds)?)
        }
    }

    /// It is only allowed to write to RAM, we shall never write to ROM
    fn store_to_ram(&mut self, addr: usize, value: u8) -> Result<()> {
        *self
            .ram
            .get_mut(addr - self.rom.len())
            .ok_or(MemoryOutOfBounds)? = value;
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
        let value = self.load_byte_from_memory(mem_addr.into())?;
        self.add(value);
        Ok(())
    }

    fn sub_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_memory(mem_addr.into())?;
        self.sub(value);
        Ok(())
    }

    fn adc_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_memory(mem_addr.into())?;
        self.adc(value);
        Ok(())
    }

    fn sbb_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_memory(mem_addr.into())?;
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

    fn and(&mut self, value: u8) {
        self.reg_a &= value;
        self.set_carry(false); // always reset carry
        self.set_zero(self.reg_a == 0);
        self.set_sign(self.reg_a >= 0x80);
        self.set_parity(self.reg_a.count_ones() % 2 == 0);
    }

    fn ani(&mut self) -> Result<()> {
        let value = self.load_d8_operand()?;
        self.and(value);
        Ok(())
    }

    fn ana_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_memory(mem_addr.into())?;
        self.and(value);
        Ok(())
    }

    fn xor(&mut self, value: u8) {
        self.reg_a ^= value;
        self.set_carry(false); // always reset carry
        self.set_zero(self.reg_a == 0);
        self.set_sign(self.reg_a >= 0x80);
        self.set_parity(self.reg_a.count_ones() % 2 == 0);
    }

    fn xri(&mut self) -> Result<()> {
        let value = self.load_d8_operand()?;
        self.xor(value);
        Ok(())
    }

    fn xra_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_memory(mem_addr.into())?;
        self.xor(value);
        Ok(())
    }

    fn or(&mut self, value: u8) {
        self.reg_a |= value;
        self.set_carry(false); // always reset carry
        self.set_zero(self.reg_a == 0);
        self.set_sign(self.reg_a >= 0x80);
        self.conditon_codes.reset_aux_carry();
        self.set_parity(self.reg_a.count_ones() % 2 == 0);
    }

    fn ori(&mut self) -> Result<()> {
        let value = self.load_d8_operand()?;
        self.or(value);
        Ok(())
    }

    fn ora_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_memory(mem_addr.into())?;
        self.or(value);
        Ok(())
    }

    fn cmp(&mut self, value: u8) {
        let _ = self.set_condition_bits(self.reg_a.into(), value.wrapping_neg().into());
        self.set_carry(self.reg_a < value);
    }

    fn cmp_m(&mut self) -> Result<()> {
        let mem_addr = construct_address((self.reg_l, self.reg_h));
        let value = self.load_byte_from_memory(mem_addr.into())?;
        self.cmp(value);
        Ok(())
    }

    fn cpi(&mut self) -> Result<()> {
        let value = self.load_d8_operand()?;
        self.cmp(value);
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
        (inr_b, reg_b, 1u8),
        (inr_c, reg_c, 1u8),
        (inr_d, reg_d, 1u8),
        (inr_e, reg_e, 1u8),
        (inr_h, reg_h, 1u8),
        (inr_l, reg_l, 1u8),
        (inr_a, reg_a, 1u8),
        (dcr_b, reg_b, 1u8.wrapping_neg()),
        (dcr_c, reg_c, 1u8.wrapping_neg()),
        (dcr_d, reg_d, 1u8.wrapping_neg()),
        (dcr_e, reg_e, 1u8.wrapping_neg()),
        (dcr_h, reg_h, 1u8.wrapping_neg()),
        (dcr_l, reg_l, 1u8.wrapping_neg()),
        (drc_a, reg_a, 1u8.wrapping_neg())
    ];

    fn inr_m(&mut self) -> Result<()> {
        let addr: usize = construct_address((self.reg_l, self.reg_h)).into();
        self.store_to_ram(addr, self.load_byte_from_memory(addr)? + 1)?;
        Ok(())
    }

    fn dcr_m(&mut self) -> Result<()> {
        let addr: usize = construct_address((self.reg_l, self.reg_h)).into();
        self.store_to_ram(addr, self.load_byte_from_memory(addr)? - 1)?;
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
        let (send, recv) = channel();
        // simulating 60Hz, a dedicated thread for timer interrupts
        // each time, we generate an interrupt for updating the vram
        thread::spawn(move || {
            let mut interrupt = 1;
            loop {
                thread::sleep(time::Duration::from_secs_f64(1.0f64 / 60.0));
                send.send(interrupt).unwrap();
                interrupt = if interrupt == 1 { 2 } else { 1 }
            }
        });
        while self.pc < self.rom.len() as u16 {
            self.execute()?;
            if let Ok(irq_no) = recv.try_recv() {
                self.pc -= 1;
                self.rst(irq_no)?;
            }
        }
        Ok(())
    }

    fn execute(&mut self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros();
        let opcode = self.load_byte_from_memory(self.pc.into())?;
        match opcode {
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
            0x07 => self.rlc(),
            0x09 => self.dad(construct_address((self.reg_c, self.reg_b))),
            0x0a => {
                self.reg_a =
                    self.load_byte_from_memory(construct_address((self.reg_c, self.reg_b)).into())?
            }
            0x0b => self.dcx_b(),
            0x0c => self.inr_c(),
            0x0d => self.dcr_c(),
            0x0e => self.reg_c = self.load_d8_operand()?,
            0x0f => self.rrc(),
            0x11 => self.load_data_into_reg_pair_d()?,
            0x12 => self.store_to_ram(
                construct_address((self.reg_e, self.reg_d)).into(),
                self.reg_a,
            )?,
            0x13 => self.inx_d(),
            0x14 => self.inr_d(),
            0x15 => self.dcr_d(),
            0x16 => self.reg_d = self.load_d8_operand()?,
            0x17 => self.ral(),
            0x19 => self.dad(construct_address((self.reg_e, self.reg_d))),
            0x1a => {
                self.reg_a =
                    self.load_byte_from_memory(construct_address((self.reg_e, self.reg_d)).into())?
            }
            0x1b => self.dcx_d(),
            0x1c => self.inr_e(),
            0x1d => self.dcr_e(),
            0x1e => self.reg_e = self.load_d8_operand()?,
            0x1f => self.rar(),
            0x21 => self.load_data_into_reg_pair_h()?,
            0x22 => self.shld()?,
            0x23 => self.inx_h(),
            0x24 => self.inr_h(),
            0x25 => self.dcr_h(),
            0x26 => self.reg_h = self.load_d8_operand()?,
            0x27 => self.daa(),
            0x29 => self.dad(construct_address((self.reg_l, self.reg_h))),
            0x2a => self.lhld()?,
            0x2b => self.dcx_h(),
            0x2c => self.inr_l(),
            0x2d => self.dcr_l(),
            0x2e => self.reg_l = self.load_d8_operand()?,
            0x2f => self.reg_a = !self.reg_a,
            0x31 => self.load_stack_pointer_from_operand()?,
            0x32 => self.sta()?,
            0x33 => self.sp += 1,
            0x34 => self.inr_m()?,
            0x35 => self.dcr_m()?,
            0x36 => {
                let imm = self.load_d8_operand()?;
                self.store_to_ram(construct_address((self.reg_l, self.reg_h)).into(), imm)?
            }
            0x37 => self.conditon_codes.set_carry(),
            0x39 => self.dad(self.sp),
            0x3a => self.lda()?,
            0x3b => self.sp -= 1,
            0x3c => self.inr_a(),
            0x3d => self.drc_a(),
            0x3e => self.reg_a = self.load_d8_operand()?,
            0x3f => self.set_carry(!self.conditon_codes.is_carry_set()),
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
            0x76 => std::process::exit(1), // HLT
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
            0xa0 => self.and(self.reg_b),
            0xa1 => self.and(self.reg_c),
            0xa2 => self.and(self.reg_d),
            0xa3 => self.and(self.reg_e),
            0xa4 => self.and(self.reg_h),
            0xa5 => self.and(self.reg_l),
            0xa6 => self.ana_m()?,
            0xa7 => self.and(self.reg_a),
            0xa8 => self.xor(self.reg_b),
            0xa9 => self.xor(self.reg_c),
            0xaa => self.xor(self.reg_d),
            0xab => self.xor(self.reg_e),
            0xac => self.xor(self.reg_h),
            0xad => self.xor(self.reg_l),
            0xae => self.xra_m()?,
            0xaf => self.xor(self.reg_a),
            0xb0 => self.or(self.reg_b),
            0xb1 => self.or(self.reg_c),
            0xb2 => self.or(self.reg_d),
            0xb3 => self.or(self.reg_e),
            0xb4 => self.or(self.reg_h),
            0xb5 => self.or(self.reg_l),
            0xb6 => self.ora_m()?,
            0xb7 => self.or(self.reg_a),
            0xb8 => self.cmp(self.reg_b),
            0xb9 => self.cmp(self.reg_c),
            0xba => self.cmp(self.reg_d),
            0xbb => self.cmp(self.reg_e),
            0xbc => self.cmp(self.reg_h),
            0xbd => self.cmp(self.reg_l),
            0xbe => self.cmp_m()?,
            0xbf => self.cmp(self.reg_a),
            0xc0 => self.ret_on_zero(!self.conditon_codes.is_zero_set())?,
            0xc1 => self.pop_b()?,
            0xc2 => self.jump_on_zero(!self.conditon_codes.is_zero_set())?,
            0xc3 => self.jmp()?,
            0xc4 => self.call_on_zero(!self.conditon_codes.is_zero_set())?,
            0xc5 => self.push_b()?,
            0xc6 => self.adi()?,
            0xc7 => self.rst(0)?,
            0xc8 => self.ret_on_zero(self.conditon_codes.is_zero_set())?,
            0xc9 => self.ret()?,
            0xca => self.jump_on_zero(self.conditon_codes.is_zero_set())?,
            0xcc => self.call_on_zero(self.conditon_codes.is_zero_set())?,
            0xcd => self.call()?,
            0xce => self.aci()?,
            0xcf => self.rst(1)?,
            0xd0 => self.ret_on_carry(!self.conditon_codes.is_carry_set())?,
            0xd1 => self.pop_d()?,
            0xd2 => self.jump_on_carry(!self.conditon_codes.is_carry_set())?,
            0xd3 => self.output()?,
            0xd4 => self.call_on_carry(!self.conditon_codes.is_carry_set())?,
            0xd5 => self.push_d()?,
            0xd6 => self.sui()?,
            0xd7 => self.rst(2)?,
            0xd8 => self.ret_on_carry(self.conditon_codes.is_carry_set())?,
            0xda => self.jump_on_carry(self.conditon_codes.is_carry_set())?,
            0xdb => self.input()?,
            0xdc => self.call_on_carry(self.conditon_codes.is_carry_set())?,
            0xde => self.sbi()?,
            0xdf => self.rst(3)?,
            0xe0 => self.ret_on_parity(!self.conditon_codes.is_parity())?,
            0xe1 => self.pop_h()?,
            0xe2 => self.jump_on_parity(!self.conditon_codes.is_parity())?,
            0xe3 => self.xthl(),
            0xe4 => self.call_on_parity(!self.conditon_codes.is_parity())?,
            0xe5 => self.push_h()?,
            0xe6 => self.ani()?,
            0xe7 => self.rst(4)?,
            0xe8 => self.ret_on_parity(self.conditon_codes.is_parity())?,
            0xe9 => self.pc = construct_address((self.reg_l, self.reg_h)) - 1,
            0xea => self.jump_on_parity(self.conditon_codes.is_parity())?,
            0xeb => self.xchg(),
            0xec => self.call_on_parity(self.conditon_codes.is_parity())?,
            0xee => self.xri()?,
            0xef => self.rst(5)?,
            0xf0 => self.ret_on_sign(!self.conditon_codes.is_sign())?,
            0xf1 => self.pop_psw()?,
            0xf2 => self.jump_on_sign(!self.conditon_codes.is_sign())?,
            0xf3 => self.interrupt_enabled = false,
            0xf4 => self.call_on_sign(!self.conditon_codes.is_sign())?,
            0xf5 => self.push_psw()?,
            0xf6 => self.ori()?,
            0xf7 => self.rst(6)?,
            0xf8 => self.ret_on_sign(self.conditon_codes.is_sign())?,
            0xf9 => self.sp = construct_address((self.reg_l, self.reg_h)),
            0xfa => self.jump_on_sign(self.conditon_codes.is_sign())?,
            0xfb => self.interrupt_enabled = true,
            0xfc => self.call_on_sign(self.conditon_codes.is_sign())?,
            0xfe => self.cpi()?,
            0xff => self.rst(7)?,
        }
        self.pc += 1;
        // execute instructions as if 2Mhz
        let time_spent = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros()
            - now;
        let circle = CLOCK_CYCLES[opcode as usize] as u64;
        if circle > time_spent as u64 {
            thread::sleep(Duration::from_micros(circle - time_spent as u64));
        }
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

    fn shld(&mut self) -> Result<()> {
        let address = construct_address(self.load_d16_operand()?);
        self.store_to_ram(address.into(), self.reg_l)?;
        self.store_to_ram((address + 1).into(), self.reg_h)?;
        self.pc += 2;
        Ok(())
    }

    fn lhld(&mut self) -> Result<()> {
        let address = construct_address(self.load_d16_operand()?);
        let lo = self.load_byte_from_memory(address.into())?;
        let hi = self.load_byte_from_memory((address + 1).into())?;
        (self.reg_l, self.reg_h) = (lo, hi);
        self.pc += 2;
        Ok(())
    }

    fn xthl(&mut self) {
        mem::swap(
            &mut self.ram[self.sp as usize - self.rom.len()],
            &mut self.reg_l,
        );
        mem::swap(
            &mut self.ram[(self.sp + 1) as usize - self.rom.len()],
            &mut self.reg_h,
        );
    }

    fn xchg(&mut self) {
        mem::swap(&mut self.reg_h, &mut self.reg_d);
        mem::swap(&mut self.reg_l, &mut self.reg_e);
    }

    fn sta(&mut self) -> Result<()> {
        let address = construct_address(self.load_d16_operand()?);
        self.store_to_ram(address.into(), self.reg_a)?;
        self.pc += 2;
        Ok(())
    }

    fn lda(&mut self) -> Result<()> {
        let address = construct_address(self.load_d16_operand()?);
        self.reg_a = self.load_byte_from_memory(address.into())?;
        self.pc += 2;
        Ok(())
    }

    pop_to_reg_pair![
        (pop_b, reg_b, reg_c),
        (pop_d, reg_d, reg_e),
        (pop_h, reg_h, reg_l)
    ];

    fn pop_psw(&mut self) -> Result<()> {
        let lo = self.load_byte_from_memory(self.sp.into())?;
        let hi = self.load_byte_from_memory((self.sp + 1).into())?;
        (*self.conditon_codes.deref_mut(), self.reg_a) = (lo, hi);
        self.sp += 2;
        Ok(())
    }

    push_to_reg_pair![
        (push_b, reg_b, reg_c),
        (push_d, reg_d, reg_e),
        (push_h, reg_h, reg_l)
    ];

    fn push_psw(&mut self) -> Result<()> {
        self.store_to_ram((self.sp - 1).into(), self.reg_a)?;
        self.store_to_ram((self.sp - 2).into(), *self.conditon_codes.deref())?;
        self.sp -= 2;
        Ok(())
    }

    fn call(&mut self) -> Result<()> {
        let pc_in_bytes = (self.pc + 2).to_be_bytes();
        self.store_to_ram((self.sp - 1).into(), pc_in_bytes[0])?;
        self.store_to_ram((self.sp - 2).into(), pc_in_bytes[1])?;
        self.sp -= 2;
        let old_pc = self.pc;
        self.pc = construct_address(self.load_d16_operand()?) - 1;
        println!(
            "call into address: {:#06x} from {:#06x}, sp = {:#06x}",
            self.pc + 1,
            old_pc,
            self.sp
        );

        #[cfg(feature = "bdos")]
        self.call_bdos(self.pc + 1)?;

        Ok(())
    }

    #[cfg(feature = "bdos")]
    fn call_bdos(&self, pc: u16) -> Result<()> {
        if pc == 0x5 || pc == 0 {
            let msg_addr = (construct_address((self.reg_e, self.reg_d)) + 3) as usize; // skipping 0CH,0DH,0AH
            println!("msg addr: {:#06x}", msg_addr);
            let msg: Vec<u8> = self
                .rom
                .iter()
                .skip(msg_addr)
                .take_while(|&&c| c as char != '$')
                .map(|c| c.to_owned())
                .collect();
            println!("{}", String::from_utf8_lossy(&msg));
            std::process::exit(0)
        }
        Ok(())
    }

    fn rst(&mut self, rst_no: u8) -> Result<()> {
        match rst_no {
            1 | 2 | 7 => {
                let pc_in_bytes = self.pc.to_be_bytes();
                self.store_to_ram((self.sp - 1).into(), pc_in_bytes[0])?;
                self.store_to_ram((self.sp - 2).into(), pc_in_bytes[1])?;
                self.sp -= 2;
                let old_pc = self.pc + 1;
                self.pc = rst_no as u16 * 8 - 1;
                println!(
                    "rst into address: {:#06x} from {:#06x}",
                    self.pc + 1,
                    old_pc
                );
            }
            _ => panic!("unsupported IRQ {rst_no}"),
        }
        Ok(())
    }

    /// TODO...
    fn output(&mut self) -> Result<()> {
        let dev_no = self.load_d8_operand()?;
        println!("accumulator {} sent to device {dev_no}", self.reg_a);
        Ok(())
    }

    /// TODO...
    fn input(&mut self) -> Result<()> {
        let dev_no = self.load_d8_operand()?;
        println!("Read from device {dev_no} and save to reg_a");
        Ok(())
    }

    fn daa(&mut self) {
        if (self.reg_a & 0xf) > 0x9 || self.conditon_codes.is_aux_carry() {
            let aux_carry = self.reg_a as u16 + 6;
            if (aux_carry & 0xf) < 0x6 {
                self.conditon_codes.set_aux_carry()
            } else {
                self.conditon_codes.reset_aux_carry()
            }
            self.reg_a = aux_carry as u8;
        }
        if (self.reg_a >> 4) > 0x9 || self.conditon_codes.is_carry_set() {
            let result = self.reg_a as u16 + (6u8 << 4) as u16;
            if result > u8::MAX.into() {
                self.conditon_codes.set_carry()
            }
            self.reg_a = result as u8;
        }
        self.set_zero(self.reg_a == 0);
        self.set_sign(self.reg_a >= 0x80);
        self.set_parity(self.reg_a.count_ones() % 2 == 0);
    }

    generate_return_on_condition![
        (ret_on_zero, is_zero_set),
        (ret_on_carry, is_carry_set),
        (ret_on_parity, is_parity_set),
        (ret_on_sign, is_sign_set)
    ];

    fn ret(&mut self) -> Result<()> {
        let addr_lo = self.load_byte_from_memory(self.sp.into())?;
        let addr_hi = self.load_byte_from_memory((self.sp + 1).into())?;
        self.pc = construct_address((addr_lo, addr_hi));
        self.sp += 2;
        println!(
            "Return to address: {:#06x}, sp = {:#06x}",
            self.pc + 1,
            self.sp
        );
        Ok(())
    }

    /// get operand parts in (lo, hi)
    fn load_d16_operand(&self) -> Result<(u8, u8)> {
        Ok((
            self.load_byte_from_memory((self.pc + 1).into())?,
            self.load_byte_from_memory((self.pc + 2).into())?,
        ))
    }

    fn load_d8_operand(&mut self) -> Result<u8> {
        let value = self.load_byte_from_memory((self.pc + 1).into())?;
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
        let old_pc = self.pc;
        self.pc = construct_address(self.load_d16_operand()?) - 1;
        println!("Jump from {:#06x} to address: {:#06x}", old_pc, self.pc + 1);
        println!(
            "Accumulator = {:#06x} condition_code: {}",
            self.reg_a, self.conditon_codes
        );
        Ok(())
    }
}

#[inline(always)]
fn construct_address((low_addr, high_addr): (u8, u8)) -> u16 {
    (high_addr as u16) << 8 | (low_addr as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_opcode_tests() {
        let dummy_rom = &vec![0; 0];
        let mut cpu = Cpu8080::new(dummy_rom);

        // test RAL & RAR
        cpu.reg_a = 0xb5;
        cpu.conditon_codes.reset_carry();
        cpu.ral();
        assert!(cpu.conditon_codes.is_carry_set());
        assert_eq!(cpu.reg_a, 0x6a);
        cpu.rar();
        assert!(!cpu.conditon_codes.is_carry_set());
        assert_eq!(cpu.reg_a, 0xb5);

        // test DAD
        cpu.reg_b = 0x33;
        cpu.reg_c = 0x9f;
        cpu.reg_h = 0xa1;
        cpu.reg_l = 0x7b;
        cpu.conditon_codes.reset_carry();
        cpu.dad(construct_address((cpu.reg_c, cpu.reg_b)));
        assert_eq!(cpu.reg_h, 0xd5);
        assert_eq!(cpu.reg_l, 0x1a);
        assert!(!cpu.conditon_codes.is_carry_set());

        // // test DAA
        cpu.reg_a = 0x9b;
        cpu.conditon_codes.reset_carry();
        cpu.conditon_codes.reset_aux_carry();
        cpu.daa();
        assert_eq!(cpu.reg_a, 0x1);
        assert!(cpu.conditon_codes.is_carry_set());
        assert!(cpu.conditon_codes.is_aux_carry());

        cpu.reg_a = 0x88;
        cpu.conditon_codes.reset_carry();
        cpu.conditon_codes.reset_aux_carry();
        cpu.add(cpu.reg_a);
        assert!(cpu.conditon_codes.is_carry_set());
        assert!(cpu.conditon_codes.is_aux_carry());
        assert_eq!(0x10, cpu.reg_a);
        cpu.daa();
        assert_eq!(0x76, cpu.reg_a);
        assert!(cpu.conditon_codes.is_carry_set());
        assert!(!cpu.conditon_codes.is_aux_carry());
    }
}
