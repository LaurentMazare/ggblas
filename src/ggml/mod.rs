trait Cpu<const ARR: usize> {
    type Unit;
    type Array;
    const STEP: usize;
    const EPR: usize;

    fn n() -> usize;
    unsafe fn zero() -> Self::Unit;
    unsafe fn zero_array() -> Self::Array;
    unsafe fn load(mem_addr: *const f32) -> Self::Unit;
    unsafe fn vec_fma(a: Self::Unit, b: Self::Unit, c: Self::Unit) -> Self::Unit;
    unsafe fn vec_reduce(x: Self::Array, y: *mut f32);
    unsafe fn from_f32(v: f32) -> Self::Unit;
    unsafe fn vec_store(mem_addr: *mut f32, a: Self::Unit);
}
trait CpuF16<const ARR: usize> {
    type Unit;
    type Array;
    const STEP: usize;
    const EPR: usize;

    fn n() -> usize;
    unsafe fn zero() -> Self::Unit;
    unsafe fn zero_array() -> Self::Array;
    unsafe fn load(mem_addr: *const f16) -> Self::Unit;
    unsafe fn vec_fma(a: Self::Unit, b: Self::Unit, c: Self::Unit) -> Self::Unit;
    unsafe fn vec_reduce(x: Self::Array, y: *mut f32);
    unsafe fn from_f32(v: f32) -> Self::Unit;
    unsafe fn vec_store(mem_addr: *mut f16, a: Self::Unit);
}
use half::f16;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[cfg(target_feature = "avx")]
pub mod avx;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[cfg(target_feature = "avx")]
pub use avx::{CurrentCpu, CurrentCpuF16};

#[cfg(any(target_arch = "wasm32"))]
#[cfg(target_feature = "simd128")]
pub mod simd128;
#[cfg(any(target_arch = "wasm32"))]
#[cfg(target_feature = "simd128")]
pub use simd128::CurrentCpu;

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
#[cfg(target_feature = "neon")]
pub mod neon;
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
#[cfg(target_feature = "neon")]
pub use neon::CurrentCpu;

#[cfg(any(
    target_feature = "neon",
    target_feature = "avx",
    target_feature = "simd128"
))]
#[inline(never)]
pub unsafe fn vec_dot_f32(a_row: *const f32, b_row: *const f32, c: *mut f32, k: usize) {
    let np = k & !(CurrentCpu::STEP - 1);

    let mut sum = CurrentCpu::zero_array();
    let mut ax = CurrentCpu::zero_array();
    let mut ay = CurrentCpu::zero_array();

    for i in (0..np).step_by(CurrentCpu::STEP) {
        for j in 0..CurrentCpu::n() {
            ax[j] = CurrentCpu::load(a_row.add(i + j * CurrentCpu::EPR));
            ay[j] = CurrentCpu::load(b_row.add(i + j * CurrentCpu::EPR));

            sum[j] = CurrentCpu::vec_fma(sum[j], ax[j], ay[j]);
        }
    }

    CurrentCpu::vec_reduce(sum, c);

    // leftovers
    for i in np..k {
        *c += *a_row.add(i) * (*b_row.add(i));
    }
}

#[cfg(any(
    target_feature = "neon",
    target_feature = "avx",
    target_feature = "simd128"
))]
pub unsafe fn vec_mad_f32(b_row: *const f32, c_row: *mut f32, v: f32, n: usize) {
    let np = n & !(CurrentCpu::STEP - 1);

    let vx = CurrentCpu::from_f32(v);
    let mut ax = CurrentCpu::zero_array();
    let mut ay = CurrentCpu::zero_array();

    for i in (0..np).step_by(CurrentCpu::STEP) {
        for j in 0..CurrentCpu::n() {
            ax[j] = CurrentCpu::load(b_row.add(i + j * CurrentCpu::EPR));
            ay[j] = CurrentCpu::load(c_row.add(i + j * CurrentCpu::EPR));
            ay[j] = CurrentCpu::vec_fma(ay[j], ax[j], vx);
            CurrentCpu::vec_store(c_row.add(i + j * CurrentCpu::EPR), ay[j]);
        }
    }

    // leftovers
    for i in np..n {
        *c_row.add(i) += *b_row.add(i) * v;
    }
}

#[cfg(not(any(
    target_feature = "neon",
    target_feature = "avx",
    target_feature = "simd128"
)))]
#[inline(never)]
pub unsafe fn vec_dot_f32(a_row: *const f32, b_row: *const f32, c: *mut f32, k: usize) {
    // leftovers
    for i in 0..k {
        *c += *a_row.add(i) * (*b_row.add(i));
    }
}

#[cfg(not(any(
    target_feature = "neon",
    target_feature = "avx",
    target_feature = "simd128"
)))]
pub unsafe fn vec_mad_f32(a_row: *const f32, c_row: *mut f32, v: f32, n: usize) {
    for i in 0..n {
        *c_row.add(i) += *a_row.add(i) * v;
    }
}

#[cfg(any(
    target_feature = "neon",
    target_feature = "avx",
    target_feature = "simd128"
))]
#[inline(never)]
pub unsafe fn vec_dot_f16(a_row: *const f16, b_row: *const f16, c: *mut f32, k: usize) {
    let mut sumf = 0.0f32;
    let np = k & !(CurrentCpuF16::STEP - 1);

    let mut sum = CurrentCpuF16::zero_array();
    let mut ax = CurrentCpuF16::zero_array();
    let mut ay = CurrentCpuF16::zero_array();

    for i in (0..np).step_by(CurrentCpuF16::STEP) {
        for j in 0..CurrentCpuF16::n() {
            ax[j] = CurrentCpuF16::load(a_row.add(i + j * CurrentCpuF16::EPR));
            ay[j] = CurrentCpuF16::load(b_row.add(i + j * CurrentCpuF16::EPR));

            sum[j] = CurrentCpuF16::vec_fma(sum[j], ax[j], ay[j]);
        }
    }

    CurrentCpuF16::vec_reduce(sum, &mut sumf);

    // leftovers
    for i in np..k {
        sumf += (*a_row.add(i)).to_f32() * (*b_row.add(i)).to_f32();
    }
    *c = sumf;
}

#[cfg(not(any(
    target_feature = "neon",
    target_feature = "avx",
    target_feature = "simd128"
)))]
#[inline(never)]
pub unsafe fn vec_dot_f16(a_row: *const f16, b_row: *const f16, c: *mut f32, k: usize) {
    // leftovers
    let mut sum = 0.0;
    for i in 0..k {
        sum += (*a_row.add(i)).to_f32() * (*b_row.add(i)).to_f32();
    }
    *c = sum;
}

pub unsafe fn f32_to_f16(x: *const f32, y: *mut f16, n: usize) {
    let mut i = 0;
    #[cfg(target_feature = "f16c")]
    #[cfg(target_feature = "avx")]
    {
        #[cfg(target_arch = "x86")]
        use core::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::*;

        while i + 7 < n {
            let x_vec: __m256 = _mm256_loadu_ps(x.add(i));
            let y_vec = _mm256_cvtps_ph(x_vec, _MM_FROUND_TO_NEAREST_INT);
            _mm_storeu_si128(y.add(i) as *mut __m128i, y_vec);
            i += 8;
        }
        while i + 3 < n {
            let x_vec: __m128 = _mm_loadu_ps(x.add(i));
            let y_vec = _mm_cvtps_ph(x_vec, _MM_FROUND_TO_NEAREST_INT);
            _mm_storel_epi64(y.add(i) as *mut __m128i, y_vec);
            i += 4;
        }
    }
    while i < n {
        *y.add(i) = f16::from_f32(*x.add(i));
        i += 1;
    }
}
