pub trait Item:
    num::Num
    + num::Zero
    + num::One
    + num::traits::NumAssign
    + num::traits::Signed
    + num::ToPrimitive
    + num::FromPrimitive
    + PartialEq
    + PartialOrd
    + Clone
    + Copy
{
    fn sqrt(self) -> Self {
        let v = Self::to_f32(&self)
            .map(|v| v.sqrt())
            .and_then(Self::from_f32);

            v.unwrap_or_else(|| Self::from_f32(0.).unwrap())
    }
}

impl Item for f64 {}

impl Item for f32 {}

impl Item for i64 {}

impl Item for i32 {}
