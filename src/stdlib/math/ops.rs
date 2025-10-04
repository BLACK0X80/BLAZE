pub const PI: f64 = 3.141592653589793;
pub const E: f64 = 2.718281828459045;
pub const TAU: f64 = 6.283185307179586;

pub fn abs(x: f64) -> f64 {
    if x < 0.0 { -x } else { x }
}

pub fn min(a: f64, b: f64) -> f64 {
    if a < b { a } else { b }
}

pub fn max(a: f64, b: f64) -> f64 {
    if a > b { a } else { b }
}

pub fn clamp(x: f64, min_val: f64, max_val: f64) -> f64 {
    if x < min_val {
        min_val
    } else if x > max_val {
        max_val
    } else {
        x
    }
}

pub fn floor(x: f64) -> f64 {
    x.floor()
}

pub fn ceil(x: f64) -> f64 {
    x.ceil()
}

pub fn round(x: f64) -> f64 {
    x.round()
}

pub fn trunc(x: f64) -> f64 {
    x.trunc()
}

pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

pub fn pow(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

pub fn exp(x: f64) -> f64 {
    x.exp()
}

pub fn ln(x: f64) -> f64 {
    x.ln()
}

pub fn log(x: f64, base: f64) -> f64 {
    x.log(base)
}

pub fn log2(x: f64) -> f64 {
    x.log2()
}

pub fn log10(x: f64) -> f64 {
    x.log10()
}

pub fn sin(x: f64) -> f64 {
    x.sin()
}

pub fn cos(x: f64) -> f64 {
    x.cos()
}

pub fn tan(x: f64) -> f64 {
    x.tan()
}

pub fn asin(x: f64) -> f64 {
    x.asin()
}

pub fn acos(x: f64) -> f64 {
    x.acos()
}

pub fn atan(x: f64) -> f64 {
    x.atan()
}

pub fn atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

