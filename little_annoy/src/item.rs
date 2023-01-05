use std::fmt::Debug;

pub trait Item:
    num::Num
    + num::Zero
    + num::One
    + num::traits::NumAssign
    + num::ToPrimitive
    + num::FromPrimitive
    + PartialOrd
    + Clone
    + Copy
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

impl Item for u64 {}

impl Item for u32 {}
