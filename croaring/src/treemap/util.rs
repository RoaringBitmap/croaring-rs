#[inline]
pub fn split(value: u64) -> (u32, u32) {
    ((value >> 32) as u32, value as u32)
}

#[inline]
pub fn join(high: u32, low: u32) -> u64 {
    (u64::from(high) << 32) | u64::from(low)
}
