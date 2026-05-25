// SPDX-FileCopyrightText: Copyright (c) 2026 Byte Facets
// SPDX-License-Identifier: MIT
pub fn pack_u64(hilo: (u32, u32)) -> u64 {
    ((hilo.0 as u64) << 32) | (hilo.1 as u64)
}

pub fn unpack_u64(value: u64) -> (u32, u32) {
    // low value explicitly extracted instead of implicitly just cast to u32
    ((value >> 32) as u32, (value & 0xFFFF_FFFF) as u32)
}

pub fn pack_u32(hilo: (u16, u16)) -> u32 {
    ((hilo.0 as u32) << 16) | (hilo.1 as u32)
}

pub fn unpack_u32(value: u32) -> (u16, u16) {
    // low value explicitly extracted instead of implicitly just cast to u16
    ((value >> 16) as u16, (value & 0xFFFF) as u16)
}

pub fn pack_u16(hilo: (u8, u8)) -> u16 {
    ((hilo.0 as u16) << 8) | (hilo.1 as u16)
}

pub fn unpack_u16(value: u16) -> (u8, u8) {
    // low value explicitly extracted instead of implicitly just cast to u8
    ((value >> 8) as u8, (value & 0xFF) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

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