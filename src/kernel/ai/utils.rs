//! Utility functions for no-std environment
//!
//! Provides common functionality missing in no-std environments
//! including string handling, math functions, and type conversions.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::fmt::Write;

/// Convert a static string to an allocated String
pub fn str_to_string(s: &str) -> String {
    String::from(s)
}

/// Simple integer square root using Newton's method
pub fn isqrt(n: u32) -> u32 {
    if n < 2 {
        return n;
    }
    
    let mut x = n;
    let mut y = (x + 1) / 2;
    
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    
    x
}

/// Simple floating point square root approximation
pub fn sqrt_approx(x: f32) -> f32 {
    if x < 0.0 {
        return 0.0; // Or handle NaN case
    }
    if x == 0.0 {
        return 0.0;
    }
    
    // Newton-Raphson method
    let mut guess = x;
    let mut last_guess = 0.0;
    let epsilon = 0.0001;
    
    while (guess - last_guess).abs() > epsilon {
        last_guess = guess;
        guess = (guess + x / guess) / 2.0;
    }
    
    guess
}

/// Simple exponential approximation using Taylor series
pub fn exp_approx(x: f32) -> f32 {
    // e^x = 1 + x + x^2/2! + x^3/3! + x^4/4! + ...
    // Use first 10 terms for reasonable accuracy
    let mut sum = 1.0;
    let mut term = 1.0;
    
    for i in 1..10 {
        term *= x / i as f32;
        sum += term;
        
        // Early exit if term becomes negligible
        if term.abs() < 0.0001 {
            break;
        }
    }
    
    sum
}

/// Round a floating point number to nearest integer
pub fn round(x: f32) -> i32 {
    if x >= 0.0 {
        (x + 0.5) as i32
    } else {
        (x - 0.5) as i32
    }
}

/// Power function for floating point (simplified)
pub fn powf(base: f32, exp: f32) -> f32 {
    // For simple integer exponents
    if exp == exp.floor() && exp.abs() < 100.0 {
        let mut result = 1.0;
        let n = exp.abs() as u32;
        
        for _ in 0..n {
            result *= base;
        }
        
        if exp < 0.0 {
            1.0 / result
        } else {
            result
        }
    } else {
        // For non-integer exponents, use exp(ln(x) * y)
        // This is a simplified approximation
        exp_approx(ln_approx(base) * exp)
    }
}

/// Natural logarithm approximation
pub fn ln_approx(x: f32) -> f32 {
    if x <= 0.0 {
        return -1000.0; // Approximation for -infinity
    }
    
    // Use the series expansion around x = 1
    // ln(x) = (x-1) - (x-1)^2/2 + (x-1)^3/3 - ...
    // This works best for x close to 1
    
    let mut n = 0;
    let mut y = x;
    
    // Normalize to [1, 2) range
    while y >= 2.0 {
        y /= 2.0;
        n += 1;
    }
    while y < 1.0 {
        y *= 2.0;
        n -= 1;
    }
    
    // Now y is in [1, 2), compute ln(y)
    let z = y - 1.0;
    let mut sum = 0.0;
    let mut term = z;
    
    for i in 1..10 {
        sum += term / i as f32;
        term *= -z;
    }
    
    // ln(x) = ln(y * 2^n) = ln(y) + n * ln(2)
    sum + n as f32 * 0.693147 // ln(2) ≈ 0.693147
}

/// Min function for f32
pub fn min_f32(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}

/// Max function for f32
pub fn max_f32(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

/// Clamp a value between min and max
pub fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Format a number to string with fixed decimal places
pub fn format_float(value: f32, decimals: usize) -> String {
    let mut s = String::new();
    
    // Handle negative
    let abs_value = if value < 0.0 {
        s.push('-');
        -value
    } else {
        value
    };
    
    // Integer part
    let integer_part = abs_value as u32;
    write!(&mut s, "{}", integer_part).unwrap();
    
    if decimals > 0 {
        s.push('.');
        
        // Fractional part
        let mut frac = abs_value - integer_part as f32;
        for _ in 0..decimals {
            frac *= 10.0;
            let digit = frac as u32;
            write!(&mut s, "{}", digit).unwrap();
            frac -= digit as f32;
        }
    }
    
    s
}

/// Generate a unique ID based on timestamp and counter
static mut ID_COUNTER: u32 = 0;

pub fn generate_unique_id(prefix: &str) -> String {
    unsafe {
        ID_COUNTER += 1;
        format!("{}_{}", prefix, ID_COUNTER)
    }
}

/// Simple hash function for strings (FNV-1a)
pub fn string_hash(s: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    
    for byte in s.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sqrt_approx() {
        assert!((sqrt_approx(4.0) - 2.0).abs() < 0.01);
        assert!((sqrt_approx(9.0) - 3.0).abs() < 0.01);
        assert!((sqrt_approx(2.0) - 1.414).abs() < 0.01);
    }
    
    #[test]
    fn test_round() {
        assert_eq!(round(3.7), 4);
        assert_eq!(round(3.3), 3);
        assert_eq!(round(-1.5), -2);
        assert_eq!(round(-1.3), -1);
    }
    
    #[test]
    fn test_string_hash() {
        let hash1 = string_hash("hello");
        let hash2 = string_hash("world");
        let hash3 = string_hash("hello");
        
        assert_eq!(hash1, hash3);
        assert_ne!(hash1, hash2);
    }
}