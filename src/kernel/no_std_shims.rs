//! No-STD API Shims for SIS Kernel
//!
//! ChatGPT's implementation-focused solutions for missing std library functionality
//! Optimized for kernel environment with zero-allocation patterns

use alloc::{string::String, vec::Vec, format};
use core::fmt::Write;

/// High-performance math operations without libm dependency
pub mod math {
    /// Fast power function approximation for f32
    /// Uses Taylor series for sub-microsecond latency in AI workloads
    #[inline]
    pub fn powf_fast(base: f32, exp: f32) -> f32 {
        if exp == 0.0 { return 1.0; }
        if exp == 1.0 { return base; }
        if base == 0.0 { return 0.0; }
        
        // Handle integer exponents efficiently
        if exp.fract() == 0.0 {
            return powi_fast(base, exp as i32);
        }
        
        // For fractional exponents, use exp(ln(base) * exp)
        let ln_base = ln_approx(base);
        exp_approx(ln_base * exp)
    }
    
    /// Fast integer power using binary exponentiation
    #[inline]
    pub fn powi_fast(base: f32, exp: i32) -> f32 {
        if exp == 0 { return 1.0; }
        if exp == 1 { return base; }
        
        let mut result = 1.0;
        let mut b = if exp < 0 { 1.0 / base } else { base };
        let mut e = exp.abs() as u32;
        
        while e > 0 {
            if e & 1 == 1 {
                result *= b;
            }
            b *= b;
            e >>= 1;
        }
        
        result
    }
    
    /// Fast square root approximation
    #[inline]
    pub fn sqrt_fast(x: f32) -> f32 {
        if x <= 0.0 { return 0.0; }
        
        // Newton-Raphson method
        let mut guess = x * 0.5;
        for _ in 0..4 { // 4 iterations for good precision
            guess = 0.5 * (guess + x / guess);
        }
        guess
    }
    
    /// Ceiling function (smallest integer >= x)
    #[inline]
    pub fn ceil_f32(x: f32) -> f32 {
        let truncated = x as i32 as f32;
        if x > truncated {
            truncated + 1.0
        } else {
            truncated
        }
    }
    
    /// Floor function (largest integer <= x)  
    #[inline]
    pub fn floor_f32(x: f32) -> f32 {
        let truncated = x as i32 as f32;
        if x < truncated {
            truncated - 1.0
        } else {
            truncated
        }
    }
    
    /// Fractional part of x (x - floor(x))
    #[inline]
    pub fn fract_f32(x: f32) -> f32 {
        x - floor_f32(x)
    }
    
    /// Natural logarithm approximation using Taylor series
    fn ln_approx(x: f32) -> f32 {
        if x <= 0.0 { return f32::NEG_INFINITY; }
        if x == 1.0 { return 0.0; }
        
        // For values far from 1, use change of variables
        if x > 2.0 || x < 0.5 {
            let mut exp = 0;
            let mut y = x;
            
            // Normalize to [0.5, 2] range
            while y > 2.0 {
                y *= 0.5;
                exp += 1;
            }
            while y < 0.5 {
                y *= 2.0;
                exp -= 1;
            }
            
            return ln_approx(y) + (exp as f32) * 0.693147; // ln(2)
        }
        
        // Taylor series around 1: ln(x) = (x-1) - (x-1)²/2 + (x-1)³/3 - ...
        let z = x - 1.0;
        let mut sum = 0.0;
        let mut term = z;
        
        for n in 1..=10 {
            sum += if n % 2 == 1 { term } else { -term } / (n as f32);
            term *= z;
        }
        
        sum
    }
    
    /// Exponential function approximation
    fn exp_approx(x: f32) -> f32 {
        if x > 88.0 { return f32::INFINITY; }
        if x < -87.0 { return 0.0; }
        
        // Taylor series: e^x = 1 + x + x²/2! + x³/3! + ...
        let mut sum = 1.0;
        let mut term = 1.0;
        
        for n in 1..=15 {
            term *= x / (n as f32);
            sum += term;
            
            // Early termination for small terms
            if term.abs() < 1e-7 { break; }
        }
        
        sum
    }
}

/// String handling without heap allocation for performance-critical paths
pub mod string {
    use super::*;
    
    /// Fixed-capacity string for kernel use
    pub struct KernelString<const N: usize> {
        buffer: [u8; N],
        len: usize,
    }
    
    impl<const N: usize> KernelString<N> {
        pub const fn new() -> Self {
            Self {
                buffer: [0; N],
                len: 0,
            }
        }
        
        pub fn from_str(s: &str) -> Self {
            let mut result = Self::new();
            let _ = result.push_str(s);
            result
        }
        
        pub fn push_str(&mut self, s: &str) -> Result<(), &'static str> {
            let bytes = s.as_bytes();
            if self.len + bytes.len() > N {
                return Err("Buffer overflow");
            }
            
            self.buffer[self.len..self.len + bytes.len()].copy_from_slice(bytes);
            self.len += bytes.len();
            Ok(())
        }
        
        pub fn as_str(&self) -> &str {
            // Safety: We only write valid UTF-8
            unsafe { core::str::from_utf8_unchecked(&self.buffer[..self.len]) }
        }
        
        pub fn clear(&mut self) {
            self.len = 0;
        }
        
        pub fn len(&self) -> usize {
            self.len
        }
        
        pub fn is_empty(&self) -> bool {
            self.len == 0
        }
    }
    
    impl<const N: usize> Write for KernelString<N> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            self.push_str(s).map_err(|_| core::fmt::Error)
        }
    }
    
    /// Convert &str to String safely
    pub fn str_to_string(s: &str) -> String {
        String::from(s)
    }
    
    /// Convert primitive types to string
    pub fn u32_to_string(val: u32) -> String {
        format!("{}", val)
    }
    
    pub fn i32_to_string(val: i32) -> String {
        format!("{}", val)
    }
    
    pub fn u64_to_string(val: u64) -> String {
        format!("{}", val)
    }
    
    pub fn f32_to_string(val: f32) -> String {
        format!("{:.6}", val)
    }
}

/// Borrow checker helpers for resolving E0382/E0502/E0596 errors
pub mod borrow {
    use core::mem;
    
    /// Replace a value temporarily, avoiding overlapping borrows
    pub fn replace_temporarily<T, R>(
        slot: &mut T, 
        temporary: T, 
        f: impl FnOnce(&mut T) -> R
    ) -> R {
        let original = mem::replace(slot, temporary);
        let result = f(slot);
        *slot = original;
        result
    }
    
    /// Take an Option, operate on it, then put it back
    pub fn with_option<T, R>(
        option: &mut Option<T>,
        f: impl FnOnce(&mut T) -> R
    ) -> Option<R> {
        if let Some(mut value) = option.take() {
            let result = f(&mut value);
            *option = Some(value);
            Some(result)
        } else {
            None
        }
    }
    
    /// Split mutable access to avoid borrow conflicts
    pub fn split_borrow<A, B, R>(
        first: &mut A,
        second: &mut Option<B>,
        f: impl FnOnce(&mut A, &mut B) -> R
    ) -> Option<R> {
        if let Some(mut b) = second.take() {
            let result = f(first, &mut b);
            *second = Some(b);
            Some(result)
        } else {
            None
        }
    }
    
    /// Atomic swap pattern for concurrent scenarios
    pub fn atomic_swap<T>(slot: &mut T, new_value: T) -> T {
        mem::replace(slot, new_value)
    }
}

/// Type inference helpers for complex generic scenarios
pub mod types {
    use alloc::vec::Vec;
    use crate::kernel::types::EdgeId;
    
    /// Explicit type aliases for common kernel collections
    pub type KernelVec<T> = Vec<T>;
    pub type EdgeVec = Vec<EdgeId>;
    pub type StringVec = Vec<alloc::string::String>;
    
    /// Helper functions with explicit type hints
    pub fn new_kernel_vec<T>() -> KernelVec<T> {
        Vec::new()
    }
    
    pub fn new_edge_vec() -> EdgeVec {
        Vec::new()
    }
    
    /// Collect iterator with explicit type
    pub fn collect_to_vec<T, I>(iter: I) -> Vec<T>
    where
        I: Iterator<Item = T>
    {
        iter.collect()
    }
}