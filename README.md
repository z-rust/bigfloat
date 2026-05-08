# bigfloat

[![Crates.io](https://img.shields.io/crates/v/bigfloat?style=flat-square)](https://crates.io/crates/bigfloat)
[![docs.rs](https://img.shields.io/docsrs/bigfloat?style=flat-square)](https://docs.rs/bigfloat)
[![License: GPLv3](https://img.shields.io/badge/license-GPLv3-blue?style=flat-square)](LICENSE)

**bigfloat** is a safe, production‑ready Rust wrapper around the
[GNU MPFR](https://www.mpfr.org/) library for arbitrary‑precision
floating‑point arithmetic.

All basic arithmetic is available through operator overloading, and many
mathematical functions (sqrt, sin, log, factorial, constants, etc.) are
exposed as free functions. Precision can be specified either by the number
of bits or by the desired number of decimal digits.

---

## ✨ Features

- **Arbitrary precision** – choose as many bits or decimal digits as you need.
- **Operator overloading** – `&x + &y`, `&x - &y`, `&x * &y`, `&x / &y`.
- **Rich math functions** – roots, powers, logs, trig, inverse trig,
  factorial, and more.
- **Constants** – `π`, `e` to any desired precision.
- **Flexible input** – functions accept both string literals and `f64`.
- **Safe Rust interface** – MPFR resources are managed automatically
  (no manual `mpfr_clear` calls).
- **Fast factorial** – uses MPFR’s `gamma` function; can compute
  `(1e6)!` in milliseconds.

---

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
bigfloat = "0.0.0"
```

### Basic arithmetic with operators

```rust
use bigfloat::*;

let x = "123.456".to_bigfloat(256);
let y = "789.012".to_bigfloat(256);
let sum = &x + &y;

assert_eq!(sum.to_string(10), "912.468");
```

### Using the free functions (default 500 decimal digits)

```rust
use bigfloat::*;

// Constants
println!("π = {}", const_pi(Some(50)));
println!("e = {}", const_e(Some(50)));

// Elementary functions
println!("√2 = {}", sqrt(2.0, Some(50)));
println!("ln(5) = {}", ln(5.0, Some(50)));
println!("10^3 = {}", pow(10.0, 3.0, Some(50)));

// Trigonometry
println!("sin(1) = {}", sin(1.0, Some(50)));
println!("cos(1) = {}", cos(1.0, Some(50)));

// Factorial – even huge arguments are fast!
println!("1000! = {}", factorial(1000.0, Some(100)));
```

---

## 📚 API Reference

### The `BigFloat` struct

| Method / trait impl            | Description                                                                      |
| ------------------------------ | -------------------------------------------------------------------------------- |
| `BigFloat::new(bits: u64)`     | Create a number with `bits` bits of precision.                                   |
| `BigFloat::new_digits(n: u32)` | Create a number with precision for at least `n` decimal digits (+32 guard bits). |
| `"…".to_bigfloat(bits)`        | Parse a decimal string into a `BigFloat`.                                        |
| `1.5.to_bigfloat(bits)`        | Convert an `f64` into a `BigFloat`.                                              |
| `num.to_string(digits)\*\*     | Format with `digits` significant digits (handles `Inf`, `-Inf`, `NaN`).          |
| `&a + &b`, `-`, `*`, `/`       | Arithmetic operators using the precision of the left operand.                    |

### Free functions (all accept `Option<u32>` for decimal digits; default 500)

**Constants**

- `const_pi(digits)` – π (pi)
- `const_e(digits)` – e (Euler’s number)

**Arithmetic**

- `add(a, b, digits)` / `sub(a, b, digits)` / `mul(a, b, digits)` / `div(a, b, digits)`
- `pow(base, exp, digits)` – `base^exp`
- `root(b, n, digits)` – `b^(1/n)`
- `sqrt(n, digits)`

**Logarithms**

- `ln(n, digits)` – natural log
- `log2(n, digits)` – base‑2 log
- `log(n, digits)` – base‑10 log

**Trigonometry**

- `sin`, `cos`, `tan` – radians
- `asin`, `acos`, `atan` – inverse trig
- `rad_to_deg`, `deg_to_rad`

**Factorial**

- `factorial(n, digits)` – `n!` computed via `Γ(n+1)`
- `ln_factorial(n, digits)` – `ln(n!)`

### The `MpfrInput` trait

```rust
pub trait MpfrInput {
    fn to_bigfloat(&self, bits: u64) -> BigFloat;
    fn to_bigfloat_digits(&self, digits: u32) -> BigFloat;
}
```

Implemented for `&str` and `f64` – so you can pass both string literals
and native floats to the free functions.

---

## 🔧 Building from Source

### Prerequisites

The crate links to native [GMP](https://gmplib.org/) and
[MPFR](https://www.mpfr.org/) libraries via
[`gmp-mpfr-sys`](https://crates.io/crates/gmp-mpfr-sys).

**Arch Linux**:

```bash
sudo pacman -S mpfr gmp
```

**Debian / Ubuntu**:

```bash
sudo apt-get install libmpfr-dev libgmp-dev
```

**macOS (Homebrew)**:

```bash
brew install mpfr gmp
```

**Windows** (via MSYS2):

```bash
pacman -S mingw-w64-x86_64-mpfr mingw-w64-x86_64-gmp
```

Then, as usual:

```bash
cargo build
cargo test
```

---

## License and AI Training

This project is licensed under the GNU General Public License v3.0 (GPLv3).

The authors of this software consider the use of this code, including its source code, documentation, and any other project artifacts, for the training of artificial intelligence (AI) systems (including but not limited to machine learning, large language models, and other AI technologies) to be creating a derivative work. As such, any entity using this code for such purposes must comply with the terms of the GPLv3. This includes, but is not limited to, making the entire source code of the AI system that uses this code available under the same GPLv3 license.

If you wish to use this code for AI training without being subject to the GPLv3, please contact the authors to negotiate a separate license.
