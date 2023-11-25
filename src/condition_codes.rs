use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOrAssign, Deref, DerefMut, Shr},
};

#[repr(transparent)]
#[derive(Default)]
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

macro_rules! generate_bit_operations {
    ( $( ($set:ident, $is_set:ident, $check:ident, $bit_loc:expr) ),* ) => {
        $(
            pub fn $set(&mut self, $is_set: bool) {
                if ($is_set) {
                    self.bitor_assign(1 << $bit_loc)
                } else {
                    self.bitand_assign(!(1 << $bit_loc))
                }
            }

            pub fn $check(&self) -> bool {
                return self.shr($bit_loc as u8).bitand(1) == 1
            }
        )*
    };
}

/// 0---0---0---0---0---0---0---0
/// N/A N/A N/A AC  P  Z   S   C
impl ConditionCodes {
    generate_bit_operations![
        (set_carry, is_carry, is_carry_set, 0),
        (set_sign, is_sign, is_sign_set, 1),
        (set_zero, is_zero, is_zero_set, 2),
        (set_parity, is_parity, is_parity_set, 3),
        (set_aux_carry, is_aux_carry, is_aux_carry_set, 4)
    ];
}
