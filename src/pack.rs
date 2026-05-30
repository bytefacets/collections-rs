// SPDX-FileCopyrightText: Copyright (c) 2026 Byte Facets
// SPDX-License-Identifier: MIT
pub const fn pack_usize(hilo: (usize, usize)) -> u128 {
    ((hilo.0 as u128) << 64) | (hilo.1 as u128)
}

pub const fn unpack_usize(value: u128) -> (usize, usize) {
    // low value explicitly extracted instead of implicitly just cast to u32
    ((value >> 64) as usize, (value & 0xFFFF_FFFF_FFFF_FFFF) as usize)
}

pub const fn pack_u64(hilo: (u32, u32)) -> u64 {
    ((hilo.0 as u64) << 32) | (hilo.1 as u64)
}

pub const fn unpack_u64(value: u64) -> (u32, u32) {
    // low value explicitly extracted instead of implicitly just cast to u32
    ((value >> 32) as u32, (value & 0xFFFF_FFFF) as u32)
}

pub const fn pack_i64(hilo: (i32, i32)) -> i64 {
    ((hilo.0 as i64) << 32) | (hilo.1 as i64)
}

pub const fn unpack_i64(value: i64) -> (i32, i32) {
    // low value explicitly extracted instead of implicitly just cast to u32
    ((value >> 32) as i32, (value & 0xFFFF_FFFF) as i32)
}

pub const fn pack_u32(hilo: (u16, u16)) -> u32 {
    ((hilo.0 as u32) << 16) | (hilo.1 as u32)
}

pub const fn unpack_u32(value: u32) -> (u16, u16) {
    // low value explicitly extracted instead of implicitly just cast to u16
    ((value >> 16) as u16, (value & 0xFFFF) as u16)
}

pub const fn pack_u16(hilo: (u8, u8)) -> u16 {
    ((hilo.0 as u16) << 8) | (hilo.1 as u16)
}

pub const fn unpack_u16(value: u16) -> (u8, u8) {
    // low value explicitly extracted instead of implicitly just cast to u8
    ((value >> 8) as u8, (value & 0xFF) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_pack_usize() {
        assert_eq!(pack_usize((0, 0)), 0);
        assert_eq!(unpack_usize(0), (0, 0));
        assert_eq!(pack_usize((usize::MAX, usize::MAX)), u128::MAX);
        assert_eq!(unpack_usize(u128::MAX), (usize::MAX, usize::MAX));
        assert_eq!(pack_usize((276765723976873, 1435191387387)), 5105426478576315568526364375164155);
        assert_eq!(unpack_usize(5105426478576315568526364375164155), (276765723976873, 1435191387387));
    }

    #[test]
    fn should_pack_u64() {
        assert_eq!(pack_u64((0, 0)), 0);
        assert_eq!(unpack_u64(0), (0, 0));
        assert_eq!(pack_u64((u32::MAX, u32::MAX)), u64::MAX);
        assert_eq!(unpack_u64(u64::MAX), (u32::MAX, u32::MAX));
        assert_eq!(pack_u64((27676572, 1435191)), 118869971606824503);
        assert_eq!(unpack_u64(118869971606824503), (27676572, 1435191));
    }
    #[test]
    fn should_pack_i64() {
        assert_eq!(pack_i64((0, 0)), 0);
        assert_eq!(unpack_i64(0), (0, 0));
        assert_eq!(pack_i64((i32::MAX, i32::MAX)), 0x7FFFFFFF_7FFFFFFF);
        assert_eq!(unpack_i64(0x7FFFFFFF_7FFFFFFF), (i32::MAX, i32::MAX));
        assert_eq!(pack_i64((27676572, 1435191)), 118869971606824503);
        assert_eq!(unpack_i64(118869971606824503), (27676572, 1435191));
    }

    #[test]
    fn should_pack_u32() {
        assert_eq!(pack_u32((0, 0)), 0);
        assert_eq!(unpack_u32(0), (0, 0));
        assert_eq!(pack_u32((u16::MAX, u16::MAX)), u32::MAX);
        assert_eq!(unpack_u32(u32::MAX), (u16::MAX, u16::MAX));
        assert_eq!(pack_u32((62546, 32159)), 4099046815);
        assert_eq!(unpack_u32(4099046815), (62546, 32159));
    }

    #[test]
    fn should_pack_u16() {
        assert_eq!(pack_u16((0, 0)), 0);
        assert_eq!(unpack_u16(0), (0, 0));
        assert_eq!(pack_u16((u8::MAX, u8::MAX)), u16::MAX);
        assert_eq!(unpack_u16(u16::MAX), (u8::MAX, u8::MAX));
        assert_eq!(pack_u16((246, 132)), 63108);
        assert_eq!(unpack_u16(63108), (246, 132));
    }
}