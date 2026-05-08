//! # bigfloat
//!
//! A safe, production-ready wrapper around MPFR for arbitrary-precision floating-point arithmetic.
//!
//! This crate provides a [`BigFloat`] type that supports high‑precision floating‑point operations.
//! All basic arithmetic (addition, subtraction, multiplication, division) is available through
//! operator overloading, and many mathematical functions (such as `sqrt`, `sin`, `log`, `factorial`,
//! constants, etc.) are exposed as free functions. Precision can be specified either by the number
//! of bits or by the desired number of decimal digits.
//!
//! # Examples
//!
//! ```
//! use bigfloat::*;
//!
//! let x = "123.456".to_bigfloat(256);
//! let y = "789.012".to_bigfloat(256);
//! let sum = &x + &y;
//! assert_eq!(sum.to_string(10), "912.468");
//!
//! let pi = const_pi(Some(50));
//! println!("π ≈ {pi}");
//! ```

use gmp_mpfr_sys::mpfr::{self, mpfr_t, rnd_t};
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::ops::{Add, Div, Mul, Sub};
use std::ptr;

/// Formats a numerical value given its base‑10 digits (as a string slice) and an exponent.
///
/// The formatting follows the rules:
/// - If the digits are exactly "0", returns "0".
/// - If the exponent is within the range `[-6, 14]`, standard decimal notation is used.
/// - Otherwise, scientific notation is used, with exactly one digit before the decimal point.
fn format_num(digits: &str, exp: i64) -> String {
    if digits == "0" {
        return "0".into();
    }

    let (sign, digits) = if let Some(rest) = digits.strip_prefix('-') {
        ("-", rest)
    } else {
        ("", digits)
    };

    const SCI_NEG_THRESH: i64 = -6;
    const SCI_POS_THRESH: i64 = 14;

    let use_scientific = !(SCI_NEG_THRESH..=SCI_POS_THRESH).contains(&exp);

    if use_scientific {
        let first = &digits[0..1];
        let rest = digits[1..].trim_end_matches('0');
        let exponent = exp - 1;
        if rest.is_empty() {
            format!("{}{}e{}", sign, first, exponent)
        } else {
            format!("{}{}.{}e{}", sign, first, rest, exponent)
        }
    } else {
        let int_len = if exp > 0 { exp as usize } else { 0 };
        let (int_part, frac_part) = if exp <= 0 {
            let leading_zeros = (-exp) as usize;
            let frac = format!("{}{}", "0".repeat(leading_zeros), digits);
            ("0".into(), frac)
        } else {
            let int = if int_len <= digits.len() {
                digits[..int_len].to_string()
            } else {
                format!("{}{}", digits, "0".repeat(int_len - digits.len()))
            };
            let frac = if int_len < digits.len() {
                digits[int_len..].to_string()
            } else {
                String::new()
            };
            (int, frac)
        };

        let frac_trimmed = frac_part.trim_end_matches('0');
        if frac_trimmed.is_empty() {
            format!("{}{}", sign, int_part)
        } else {
            format!("{}{}.{}", sign, int_part, frac_trimmed)
        }
    }
}

/// An arbitrary‑precision floating‑point number backed by an MPFR variable.
///
/// The precision (in bits) is set at creation time and cannot be changed afterwards.
/// All arithmetic operations use the precision of the left‑hand operand (for binary operators)
/// or the precision of the operands (for the free functions).
pub struct BigFloat {
    inner: mpfr_t,
}

impl BigFloat {
    /// Constructs a new `BigFloat` with the given precision in **bits**.
    ///
    /// The internal MPFR variable is initialized with the exact bit‑precision requested.
    pub fn new(bits: u64) -> Self {
        unsafe {
            let mut v = MaybeUninit::uninit();
            mpfr::init2(v.as_mut_ptr(), bits as i64);
            BigFloat {
                inner: v.assume_init(),
            }
        }
    }

    /// Constructs a new `BigFloat` with enough bits to hold at least `digits` decimal digits
    /// of precision, plus a small safety margin.
    ///
    /// The conversion factor used is `digits * log2(10) ≈ digits * 3.3219280948873626`,
    /// rounded up, with an additional 32 bits.
    pub fn new_digits(digits: u32) -> Self {
        unsafe {
            let mut v = MaybeUninit::uninit();
            mpfr::init2(
                v.as_mut_ptr(),
                (digits as f64 * 3.3219280948873626).ceil() as i64 + 32,
            );
            BigFloat {
                inner: v.assume_init(),
            }
        }
    }

    /// Parses a decimal string into a `BigFloat` with the specified bit‑precision.
    ///
    /// # Panics
    ///
    /// Panics if the string is not a valid floating‑point number (including `Inf`, `NaN`, etc.).
    pub fn from_str(s: &str, bits: u64) -> Self {
        let mut num = BigFloat::new(bits);
        unsafe {
            let c = CString::new(s).unwrap();
            if mpfr::set_str(&mut num.inner, c.as_ptr(), 10, rnd_t::RNDN) != 0 {
                mpfr::clear(&mut num.inner);
                panic!("Bad number: {}", s);
            }
        }
        num
    }

    /// Parses a decimal string into a `BigFloat` with enough precision for `digits`.
    ///
    /// This is equivalent to calling [`from_str`](BigFloat::from_str) with `bits` computed
    /// from `digits` as in [`new_digits`](BigFloat::new_digits).
    ///
    /// # Panics
    ///
    /// Panics if the string is not a valid floating‑point number.
    pub fn from_str_digits(s: &str, digits: u32) -> Self {
        let mut num = BigFloat::new_digits(digits);
        unsafe {
            let c = CString::new(s).unwrap();
            if mpfr::set_str(&mut num.inner, c.as_ptr(), 10, rnd_t::RNDN) != 0 {
                mpfr::clear(&mut num.inner);
                panic!("Bad number: {}", s);
            }
        }
        num
    }

    /// Returns a base‑10 representation with `digits` significant digits.
    ///
    /// The output format is similar to `printf("%.*Re", digits-1, value)` in C, but
    /// follows standard Rust formatting conventions (e.g., no trailing zeros after the
    /// decimal point when unnecessary).
    ///
    /// Special values are rendered as `"Inf"`, `"-Inf"`, or `"NaN"`.
    pub fn to_string(&self, digits: u32) -> String {
        let mut exp: mpfr::exp_t = 0;
        unsafe {
            let ptr = mpfr::get_str(
                ptr::null_mut(),
                &mut exp,
                10,
                digits as usize,
                &self.inner,
                rnd_t::RNDN,
            );
            if ptr.is_null() {
                return "Error".into();
            }
            let cstr = CStr::from_ptr(ptr);
            let d = cstr.to_str().unwrap_or("??");

            if d.contains("@Inf@") || d.contains("@NaN@") {
                mpfr::free_str(ptr);
                return if d.contains("-") {
                    "-Inf".to_string()
                } else if d.contains("NaN") {
                    "NaN".to_string()
                } else {
                    "Inf".to_string()
                };
            }

            let result = format_num(d, exp);
            mpfr::free_str(ptr);
            result
        }
    }
}

impl Drop for BigFloat {
    fn drop(&mut self) {
        unsafe {
            mpfr::clear(&mut self.inner);
        }
    }
}

// Operator overloads ---------------------------------------------------------

impl<'a> Add<&'a BigFloat> for &'a BigFloat {
    type Output = BigFloat;

    fn add(self, rhs: &'a BigFloat) -> BigFloat {
        let prec = unsafe { mpfr::get_prec(&self.inner) as u64 };
        let mut result = BigFloat::new(prec);
        unsafe {
            mpfr::add(&mut result.inner, &self.inner, &rhs.inner, rnd_t::RNDN);
        }
        result
    }
}

impl<'a> Sub<&'a BigFloat> for &'a BigFloat {
    type Output = BigFloat;
    fn sub(self, rhs: &'a BigFloat) -> BigFloat {
        let prec = unsafe { mpfr::get_prec(&self.inner) as u64 };
        let mut result = BigFloat::new(prec);
        unsafe {
            mpfr::sub(&mut result.inner, &self.inner, &rhs.inner, rnd_t::RNDN);
        }
        result
    }
}

impl<'a> Mul<&'a BigFloat> for &'a BigFloat {
    type Output = BigFloat;
    fn mul(self, rhs: &'a BigFloat) -> BigFloat {
        let prec = unsafe { mpfr::get_prec(&self.inner) as u64 };
        let mut result = BigFloat::new(prec);
        unsafe {
            mpfr::mul(&mut result.inner, &self.inner, &rhs.inner, rnd_t::RNDN);
        }
        result
    }
}

impl<'a> Div<&'a BigFloat> for &'a BigFloat {
    type Output = BigFloat;
    fn div(self, rhs: &'a BigFloat) -> BigFloat {
        let prec = unsafe { mpfr::get_prec(&self.inner) as u64 };
        let mut result = BigFloat::new(prec);
        unsafe {
            mpfr::div(&mut result.inner, &self.inner, &rhs.inner, rnd_t::RNDN);
        }
        result
    }
}

// Trait to abstract input types for MPFR conversion --------------------------

/// Trait for types that can be converted into a [`BigFloat`].
///
/// This is implemented for `&str` and `f64`, allowing the free arithmetic and
/// mathematical functions to accept both numeric string literals and native floats
/// as arguments.
pub trait MpfrInput {
    /// Converts the value to a `BigFloat` with the given bit‑precision.
    fn to_bigfloat(&self, bits: u64) -> BigFloat;
    /// Converts the value to a `BigFloat` with a precision sufficient for `digits`
    /// decimal digits.
    fn to_bigfloat_digits(&self, digits: u32) -> BigFloat;
}

impl MpfrInput for &str {
    fn to_bigfloat(&self, bits: u64) -> BigFloat {
        BigFloat::from_str(self, bits)
    }

    fn to_bigfloat_digits(&self, digits: u32) -> BigFloat {
        BigFloat::from_str(
            self,
            (digits as f64 * 3.3219280948873626).ceil() as u64 + 32,
        )
    }
}

impl MpfrInput for f64 {
    fn to_bigfloat(&self, bits: u64) -> BigFloat {
        let mut num = BigFloat::new(bits);
        unsafe {
            // Direct binary conversion preserves full f64 precision
            mpfr::set_d(&mut num.inner, *self, rnd_t::RNDN);
        }
        num
    }

    fn to_bigfloat_digits(&self, digits: u32) -> BigFloat {
        let mut num = BigFloat::new((digits as f64 * 3.3219280948873626).ceil() as u64 + 32);
        unsafe {
            // Direct binary conversion preserves full f64 precision
            mpfr::set_d(&mut num.inner, *self, rnd_t::RNDN);
        }
        num
    }
}

// Free arithmetic functions --------------------------------------------------

/// Returns the sum of two values, with `digits` decimal digits of precision (default 500).
///
/// # Example
/// ```
/// # use bigfloat::add;
/// assert_eq!(add("0.1", "0.2", Some(30)), "0.3");
/// ```
pub fn add<T: MpfrInput>(a: T, b: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let x = a.to_bigfloat_digits(digits);
    let y = b.to_bigfloat_digits(digits);

    let x = &x + &y;
    x.to_string(digits)
}

/// Returns the difference `a - b`, with `digits` decimal digits of precision (default 500).
pub fn sub<T: MpfrInput>(a: T, b: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let x = a.to_bigfloat_digits(digits);
    let y = b.to_bigfloat_digits(digits);

    let x = &x - &y;
    x.to_string(digits)
}

/// Returns the product `a * b`, with `digits` decimal digits of precision (default 500).
pub fn mul<T: MpfrInput>(a: T, b: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let x = a.to_bigfloat_digits(digits);
    let y = b.to_bigfloat_digits(digits);

    let x = &x * &y;
    x.to_string(digits)
}

/// Returns the quotient `a / b`, with `digits` decimal digits of precision (default 500).
pub fn div<T: MpfrInput>(a: T, b: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let x = a.to_bigfloat_digits(digits);
    let y = b.to_bigfloat_digits(digits);

    let x = &x / &y;
    x.to_string(digits)
}

/// Returns the factorial of `n`, `n!`, computed via `Γ(n+1)`.
///
/// This uses MPFR’s `gamma` function for efficiency – it can handle huge arguments
/// (e.g. `1e6`) almost instantly.
pub fn factorial<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);
    let mut n_plus_1 = BigFloat::new_digits(digits);

    unsafe {
        // Create n + 1
        let one = BigFloat::from_str_digits("1", digits);
        mpfr::add(&mut n_plus_1.inner, &input.inner, &one.inner, rnd_t::RNDN);
        // n! = Γ(n+1)
        // MPFR uses asymptotic expansions & binary splitting internally.
        // Runs in milliseconds instead of years.
        mpfr::gamma(&mut result.inner, &n_plus_1.inner, rnd_t::RNDN);
    }

    result.to_string(digits)
}

/// Returns the natural logarithm of the factorial (`ln(n!)`).
pub fn ln_factorial<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);
    let mut n_plus_1 = BigFloat::new_digits(digits);

    unsafe {
        let one = BigFloat::from_str_digits("1", digits);
        mpfr::add(&mut n_plus_1.inner, &input.inner, &one.inner, rnd_t::RNDN);

        // ln(Γ(n+1)) = ln(n!)
        mpfr::lngamma(&mut result.inner, &n_plus_1.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

// Constants ------------------------------------------------------------------

/// Returns π (pi) to `d` decimal digits of precision (default 500).
pub fn const_pi(d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::const_pi(&mut result.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

/// Returns *e* (Euler’s number) to `d` decimal digits of precision (default 500).
pub fn const_e(d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        let one = BigFloat::from_str_digits("1", digits);
        mpfr::exp(&mut result.inner, &one.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

// Elementary functions -------------------------------------------------------

/// Returns the square root of `n` with `d` decimal digits of precision (default 500).
pub fn sqrt<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::sqrt(&mut result.inner, &input.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

/// Returns the `n`‑th root of `b` (i.e., `b^(1/n)`) with `d` decimal digits of precision.
///
/// # Example
/// ```
/// # use bigfloat::root;
/// assert_eq!(root(27.0, 3.0, Some(10)), "3");
/// ```
pub fn root<T: MpfrInput>(b: T, n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let b = b.to_bigfloat_digits(digits);
    let n = n.to_bigfloat_digits(digits);

    let mut one_over_n = BigFloat::new_digits(digits);
    let mut result = BigFloat::new_digits(digits);

    unsafe {
        let one = BigFloat::from_str_digits("1", digits);
        mpfr::div(&mut one_over_n.inner, &one.inner, &n.inner, rnd_t::RNDN);
        mpfr::pow(&mut result.inner, &b.inner, &one_over_n.inner, rnd_t::RNDN);
    }

    result.to_string(digits)
}

/// Returns the result of raising `b` to the power `p` (`b^p`).
pub fn pow<T: MpfrInput>(b: T, p: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let b = b.to_bigfloat_digits(digits);
    let p = p.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::pow(&mut result.inner, &b.inner, &p.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

/// Returns the ln of `n` (`ln(n)`).
pub fn ln<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::log(&mut result.inner, &input.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

/// Returns the base‑2 logarithm of `n` (`log2(n)`).
pub fn log2<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::log2(&mut result.inner, &input.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

/// Returns the base‑10 logarithm of `n` (`log10(n)`).
pub fn log<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::log10(&mut result.inner, &input.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

// Trigonometric functions ----------------------------------------------------

/// Converts radians to degrees.
pub fn rad_to_deg<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let n = n.to_bigfloat_digits(digits);

    let mut num_pi = BigFloat::new_digits(digits);
    let result;

    unsafe {
        mpfr::const_pi(&mut num_pi.inner, rnd_t::RNDN);

        let n180 = BigFloat::from_str_digits("180", digits);
        let n180_over_pi = &n180 / &num_pi;

        result = &n * &n180_over_pi;
    }

    result.to_string(digits)
}

/// Converts degrees to radians.
pub fn deg_to_rad<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let n = n.to_bigfloat_digits(digits);

    let mut num_pi = BigFloat::new_digits(digits);
    let result;

    unsafe {
        mpfr::const_pi(&mut num_pi.inner, rnd_t::RNDN);

        let n180 = BigFloat::from_str_digits("180", digits);
        let pi_over_n180 = &num_pi / &n180;

        result = &n * &pi_over_n180;
    }

    result.to_string(digits)
}

/// Returns the sine of `n` radians.
pub fn sin<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::sin(&mut result.inner, &input.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

/// Returns the arcsine (inverse sine) of `n` radians.
pub fn asin<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::asin(&mut result.inner, &input.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

/// Returns the cosine of `n` radians.
pub fn cos<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::cos(&mut result.inner, &input.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

/// Returns the arccosine (inverse cosine) of `n` radians.
pub fn acos<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::acos(&mut result.inner, &input.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

/// Returns the tangent of `n` radians.
pub fn tan<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::tan(&mut result.inner, &input.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}

/// Returns the arctangent (inverse tangent) of `n` radians.
pub fn atan<T: MpfrInput>(n: T, d: Option<u32>) -> String {
    let digits = d.unwrap_or(500);
    let input = n.to_bigfloat_digits(digits);

    let mut result = BigFloat::new_digits(digits);

    unsafe {
        mpfr::atan(&mut result.inner, &input.inner, rnd_t::RNDN);
    }
    result.to_string(digits)
}
