#![feature(cfg_target_feature, target_feature)]
#![feature(test)]

#[macro_use]
extern crate stdsimd;
mod simd;
use stdsimd::vendor;
use stdsimd::simd::i32x4;

fn main() {
    let a = i32x4::new(1, 2, 3, 4);
    let b = i32x4::splat(10);
    assert_eq!(b, i32x4::new(10, 10, 10, 10));
    let c = a + b;
    assert_eq!(c, i32x4::new(11, 12, 13, 14));
    assert_eq!(sum_portable(b), 40);
    assert_eq!(sum_ct(b), 40);
    assert_eq!(sum_rt(b), 40);
}

// Sums the elements of the vector.
fn sum_portable(x: i32x4) -> i32 {
    let mut r = 0;
    for i in 0..4 {
        r += x.extract(i);
    }
    r
}

// Sums the elements of the vector using SSE2 instructions.
// This function is only safe to call if the CPU where the
// binary runs supports SSE2.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature = "+sse2"]
unsafe fn sum_sse2(x: i32x4) -> i32 {
    let x = vendor::_mm_add_epi32(x, vendor::_mm_srli_si128(x.into(), 8).into());
    let x = vendor::_mm_add_epi32(x, vendor::_mm_srli_si128(x.into(), 4).into());
    vendor::_mm_cvtsi128_si32(x)
}

// Uses the SSE2 version if SSE2 is enabled for all target
// CPUs at compile-time (does not perform any run-time
// feature detection).
fn sum_ct(x: i32x4) -> i32 {
    #[cfg(all(any(target_arch = "x86_64", target_arch = "x86"), target_feature = "sse2"))]
    {
        // This function is only available for x86/x86_64 targets,
        // and is only safe to call it if the target supports SSE2
        unsafe { sum_sse2(x) }
    }
    #[cfg(not(all(any(target_arch = "x86_64", target_arch = "x86"), target_feature = "sse2")))]
    {
        sum_portable(x)
    }
}

// Detects SSE2 at run-time, and uses a SIMD intrinsic if enabled.
fn sum_rt(x: i32x4) -> i32 {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        // If SSE2 is not enabled at compile-time, this
        // detects whether SSE2 is available at run-time:
        if cfg_feature_enabled!("sse2") {
            return unsafe { sum_sse2(x) };
        }
    }
    sum_portable(x)
}
