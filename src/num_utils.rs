// SPDX-FileCopyrightText: Copyright (c) 2025 Byte Facets
// SPDX-License-Identifier: MIT

pub fn next_power_of_2(n: usize) -> usize {
    if n == 0 {
        1
    } else {
        1 << (usize::BITS - (n - 1).leading_zeros())
    }
}
