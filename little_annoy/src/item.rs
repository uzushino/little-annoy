use num::{traits::NumAssign, FromPrimitive, One, Signed, ToPrimitive, Zero};

pub trait Item:
    Zero + One + NumAssign + ToPrimitive + FromPrimitive + PartialOrd + Signed + Clone + Copy
{
    fn sqrt(self) -> Self {
        let v = Self::to_f64(&self)
            .map(|v| v.sqrt())
            .and_then(Self::from_f64);
        v.unwrap_or_else(|| Self::zero())
    }
}

impl Item for isize {}

impl Item for i8 {}

impl Item for i16 {}

impl Item for f64 {}

impl Item for f32 {}

impl Item for i64 {}

impl Item for i32 {}
