use std::ops::{BitAnd, BitAndAssign, BitOrAssign, Deref, DerefMut, Shr};

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

impl ConditionCodes {
    pub fn set_carry(&mut self) {
        self.bitor_assign(1)
    }

    pub fn reset_carry(&mut self) {
        self.bitand_assign(!1)
    }

    pub fn is_carry_set(&self) -> bool {
        self.bitand(1) == 1
    }

    pub fn set_sign(&mut self) {
        self.bitor_assign(2)
    }

    pub fn reset_sign(&mut self) {
        self.bitand_assign(!2)
    }

    pub fn is_sign(&self) -> bool {
        self.shr(1u8).bitand(1) == 1
    }

    pub fn set_zero(&mut self) {
        self.bitor_assign(4)
    }

    pub fn reset_zero(&mut self) {
        self.bitand_assign(!4)
    }

    pub fn is_zero_set(&self) -> bool {
        self.shr(2u8).bitand(1) == 1
    }

    pub fn set_parity(&mut self) {
        self.bitor_assign(8)
    }

    pub fn reset_parity(&mut self) {
        self.bitand_assign(!8)
    }

    pub fn is_parity(&self) -> bool {
        self.shr(3u8).bitand(1) == 1
    }

    pub fn set_aux_carry(&mut self) {
        self.bitor_assign(16)
    }

    pub fn reset_aux_carry(&mut self) {
        self.bitand_assign(!16)
    }

    pub fn is_aux_carry(&self) -> bool {
        self.shr(4u8).bitand(1) == 1
    }
}
