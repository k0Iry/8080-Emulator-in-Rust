use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOrAssign, Deref, DerefMut, Shr},
};

#[repr(transparent)]
#[derive(Default, Debug)]
pub struct ConditionCodes(u8);

impl Deref for ConditionCodes {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ConditionCodes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for ConditionCodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "AC = {}, P = {}, Z = {}, S = {}, C = {}",
            self.shr(4u8).bitand(1),
            self.shr(3u8).bitand(1),
            self.shr(2u8).bitand(1),
            self.shr(1u8).bitand(1),
            self.bitand(1)
        )
    }
}

macro_rules! generate_set_bit {
    ( $( ($set:ident, $bit:expr) ),* ) => {
        $(
            pub fn $set(&mut self) {
                self.bitor_assign($bit)
            }
        )*
    };
}

macro_rules! generate_reset_bit {
    ( $( ($reset:ident, $bit:expr) ),* ) => {
        $(
            pub fn $reset(&mut self) {
                self.bitand_assign(!$bit)
            }
        )*
    };
}

macro_rules! generate_check_bit {
    ( $( ($check:ident, $bit_loc:expr) ),* ) => {
        $(
            pub fn $check(&self) -> bool {
                self.shr($bit_loc as u8).bitand(1) == 1
            }
        )*
    };
}

/// 0---0---0---0---0---0---0---0
/// N/A N/A N/A AC  P  Z   S   C
impl ConditionCodes {
    generate_set_bit![
        (set_carry, 1),
        (set_sign, 2),
        (set_zero, 4),
        (set_parity, 8),
        (set_aux_carry, 16)
    ];

    generate_reset_bit![
        (reset_carry, 1),
        (reset_sign, 2),
        (reset_zero, 4),
        (reset_parity, 8),
        (reset_aux_carry, 16)
    ];

    generate_check_bit![
        (is_carry_set, 0),
        (is_sign, 1),
        (is_zero_set, 2),
        (is_parity, 3),
        (is_aux_carry, 4)
    ];
}
