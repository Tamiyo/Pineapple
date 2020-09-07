/// Rust prevents me from hashing floats directly, so I've defined the DistanceF64
/// struct to convert each float into it's integer form.
///
/// TODO :- Look into if this is actually efficient.
///         It is lazy and it allows me to Copy constants :)
///
/// See https://stackoverflow.com/questions/39638363/how-can-i-use-a-hashmap-with-f64-as-key-in-rust
use std::cmp;
use std::ops;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct DistanceF64 {
    mantissa: u64,
    exponent: i16,
    sign: i8,
}

impl DistanceF64 {
    pub fn new(mantissa: u64, exponent: i16, sign: i8) -> Self {
        DistanceF64 {
            mantissa,
            exponent,
            sign,
        }
    }
}

impl From<f64> for DistanceF64 {
    fn from(item: f64) -> Self {
        let bits: u64 = item.to_bits();
        let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
        let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
        let mantissa = if exponent == 0 {
            (bits & 0x000f_ffff_ffff_ffff) << 1
        } else {
            (bits & 0x000f_ffff_ffff_ffff) | 0x0010_0000_0000_0000
        };

        exponent -= 1023 + 52;

        DistanceF64 {
            mantissa,
            exponent,
            sign,
        }
    }
}

impl Into<f64> for DistanceF64 {
    fn into(self) -> f64 {
        (self.sign as f64) * (self.mantissa as f64) * (2f64.powf(self.exponent as f64))
    }
}

impl Into<f64> for &DistanceF64 {
    fn into(self) -> f64 {
        (self.sign as f64) * (self.mantissa as f64) * (2f64.powf(self.exponent as f64))
    }
}

impl ops::Add<DistanceF64> for DistanceF64 {
    type Output = DistanceF64;

    fn add(self, _rhs: DistanceF64) -> DistanceF64 {
        DistanceF64::from(Into::<f64>::into(self) + Into::<f64>::into(_rhs))
    }
}

impl ops::Sub<DistanceF64> for DistanceF64 {
    type Output = DistanceF64;

    fn sub(self, _rhs: DistanceF64) -> DistanceF64 {
        DistanceF64::from(Into::<f64>::into(self) - Into::<f64>::into(_rhs))
    }
}

impl ops::Mul<DistanceF64> for DistanceF64 {
    type Output = DistanceF64;

    fn mul(self, _rhs: DistanceF64) -> DistanceF64 {
        DistanceF64::from(Into::<f64>::into(self) * Into::<f64>::into(_rhs))
    }
}

impl ops::Div<DistanceF64> for DistanceF64 {
    type Output = DistanceF64;

    fn div(self, _rhs: DistanceF64) -> DistanceF64 {
        DistanceF64::from(Into::<f64>::into(self) / Into::<f64>::into(_rhs))
    }
}

impl ops::Rem<DistanceF64> for DistanceF64 {
    type Output = DistanceF64;

    fn rem(self, _rhs: DistanceF64) -> DistanceF64 {
        DistanceF64::from(Into::<f64>::into(self) % Into::<f64>::into(_rhs))
    }
}

impl cmp::PartialOrd for DistanceF64 {
    fn partial_cmp(&self, other: &DistanceF64) -> Option<cmp::Ordering> {
        let n1 = Into::<f64>::into(self);
        let n2 = Into::<f64>::into(other);
        n1.partial_cmp(&n2)
    }
}

impl std::fmt::Debug for DistanceF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Into::<f64>::into(self))
    }
}
