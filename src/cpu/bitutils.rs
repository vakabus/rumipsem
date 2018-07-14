//! Bit operation utilities

pub fn get_opcode(instruction: u32) -> u32 {
    (instruction & 0xFC_00_00_00) >> 26
}

pub fn get_rs(instruction: u32) -> u32 {
    (instruction & 0x03_E0_00_00) >> 21
}

pub fn get_rt(instruction: u32) -> u32 {
    (instruction & 0x00_1F_00_00) >> 16
}

pub fn get_rd(instruction: u32) -> u32 {
    (instruction & 0x00_00_F8_00) >> 11
}

pub fn get_shift(instruction: u32) -> u32 {
    (instruction & 0x00_00_07_C0) >> 6
}

pub fn get_funct(instruction: u32) -> u32 {
    (instruction & 0x00_00_00_3F) >> 0
}

pub fn get_offset(instruction: u32) -> u16 {
    (instruction & 0x00_00_FF_FF) as u16
}

pub fn add_signed_offset(word: u32, offset: u16) -> u32 {
    word.overflowing_add(((offset as i16) as i32) as u32).0
}

pub fn add_to_upper_bits(word: u32, immediate: u16) -> u32 {
    ((word as i32) + (((immediate as u32) << 16) as i32)) as u32
}

pub fn sign_extend(word: u32, length: u8) -> i32 {
    assert!(length < 32);
    ((word as i32) << (32 - length)) >> (32 - length)
    //(word | (0xFF_FF_FF_FF ^ (((word & (1 << (length - 1))) << 1) - 1))) as i32
}

#[test]
fn test_apply_offset() {
    assert_eq!(add_signed_offset(0, 10), 10);
    assert_eq!(add_signed_offset(65535, 10), 65545);
    assert_eq!(add_signed_offset(65535, 65535), 65534);
    assert_eq!(add_signed_offset(0xFF_FF_FF_00, 0xFF), 0xFF_FF_FF_FF);
    assert_eq!(
        add_signed_offset(0xFF_FF_FF_FF, 0x80_00),
        0xFF_FF_FF_FF - (65535 / 2) - 1
    );
}

#[test]
fn test_sign_extend() {
    assert_eq!(sign_extend(0xFF, 8), -1);
    assert_eq!(sign_extend(0x00_FF_FF_FF, 24), -1);
}

#[test]
fn test_add_to_upper_bits() {
    assert_eq!(add_to_upper_bits(0x0000_0000, 0x7F_FF), 0x7F_FF_00_00);
    assert_eq!(add_to_upper_bits(0x0001_0001, 0x0001), 0x0002_0001);
    assert_eq!(add_to_upper_bits(0x7FFF_0001, 0x0000), 0x7FFF_0001);
    assert_eq!(add_to_upper_bits(0x7FFF_0001, 0xFFFF), 0x7FFE_0001);
}
