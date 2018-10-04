#[inline(always)]
pub fn check_bit(val: u8, b: u8) -> bool {
    val & (1 << b) > 0
}
