#![feature(stdsimd)]
extern crate core;

use core::arch::x86_64::__m128i;
use core::arch::x86_64::_mm_cmpeq_epi8;
use core::arch::x86_64::_mm_movemask_epi8;
use core::arch::x86_64::_mm_set1_epi8;
use core::simd::i8x16;
use core::simd::FromBits;
fn main() {
    unsafe {
        let raw_node_key = i8x16::new(8, 12, 43, 5, 6, 4, 3, 44, 35, 74, 37, 35, 19, 74, 69, 54);
        let node_key: __m128i = FromBits::from_bits(raw_node_key);
        let key = _mm_set1_epi8(8);
        let cmp = _mm_cmpeq_epi8(key, node_key);
        let mask = (1 << 16) - 1;
        let result = _mm_movemask_epi8(cmp) & mask;
        println!("result {:?}", result);
        println!("{:?} ,{}", _mm_movemask_epi8(cmp), result & mask);
    }
}
