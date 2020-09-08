/// Rust prevents me from hashing floats directly, so I've defined the DistanceF32
/// struct to convert each float into it's integer form.
///
/// TODO :- Look into if this is actually efficient.
///         It is lazy and it allows me to Copy constants :)
///
/// See https://stackoverflow.com/questions/39638363/how-can-i-use-a-hashmap-with-f64-as-key-in-rust
use std::cmp;
use std::ops;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct DistanceF32 {
    mantissa: u32,
    exponent: i8,
    sign: i8,
}

impl DistanceF32 {
    pub fn new(mantissa: u32, exponent: i8, sign: i8) -> Self {
        DistanceF32 {
            mantissa,
            exponent,
            sign,
        }
    }
}

impl From<f32> for DistanceF32 {
    fn from(item: f32) -> Self {
        let bits: u32 = item.to_bits();
        let sign: i8 = if bits >> 31 == 0 { 1 } else { -1 };
        let mut exponent: i8 = ((bits >> 23) & 0xFF) as i8;
        let mantissa = if exponent == 0 {
            (bits & 0x000f_ffff) << 1
        } else {
            (bits & 0x000f_ffff) | 0x0010_0000
        };

        exponent -= 16 + 23;

        DistanceF32 {
            mantissa,
            exponent,
            sign,
        }
    }
}

impl Into<f32> for DistanceF32 {
    fn into(self) -> f32 {
        (self.sign as f32) * (self.mantissa as f32) * (2f32.powf(self.exponent as f32))
    }
}

impl Into<f32> for &DistanceF32 {
    fn into(self) -> f32 {
        (self.sign as f32) * (self.mantissa as f32) * (2f32.powf(self.exponent as f32))
    }
}

impl ops::Add<DistanceF32> for DistanceF32 {
    type Output = DistanceF32;

    fn add(self, _rhs: DistanceF32) -> DistanceF32 {
        DistanceF32::from(Into::<f32>::into(self) + Into::<f32>::into(_rhs))
    }
}

impl ops::Sub<DistanceF32> for DistanceF32 {
    type Output = DistanceF32;

    fn sub(self, _rhs: DistanceF32) -> DistanceF32 {
        DistanceF32::from(Into::<f32>::into(self) - Into::<f32>::into(_rhs))
    }
}

impl ops::Mul<DistanceF32> for DistanceF32 {
    type Output = DistanceF32;

    fn mul(self, _rhs: DistanceF32) -> DistanceF32 {
        DistanceF32::from(Into::<f32>::into(self) * Into::<f32>::into(_rhs))
    }
}

impl ops::Div<DistanceF32> for DistanceF32 {
    type Output = DistanceF32;

    fn div(self, _rhs: DistanceF32) -> DistanceF32 {
        DistanceF32::from(Into::<f32>::into(self) / Into::<f32>::into(_rhs))
    }
}

impl ops::Rem<DistanceF32> for DistanceF32 {
    type Output = DistanceF32;

    fn rem(self, _rhs: DistanceF32) -> DistanceF32 {
        DistanceF32::from(Into::<f32>::into(self) % Into::<f32>::into(_rhs))
    }
}

impl cmp::PartialOrd for DistanceF32 {
    fn partial_cmp(&self, other: &DistanceF32) -> Option<cmp::Ordering> {
        let n1 = Into::<f32>::into(self);
        let n2 = Into::<f32>::into(other);
        n1.partial_cmp(&n2)
    }
}

impl std::fmt::Debug for DistanceF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Into::<f32>::into(self))
    }
}
