extern "C" {
    pub fn read_unaligned_volatile_u8(_: *const u8) -> u8;
    pub fn write_unaligned_volatile_u8(_: *mut u8, _: u8);
}
