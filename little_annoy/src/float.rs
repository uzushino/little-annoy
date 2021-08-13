pub trait Float:
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
        let v = Self::to_f32(&self).unwrap();
        let f = v.sqrt();
        Self::from_f32(f).unwrap()
    }
}

impl Float for f64 {}

impl Float for f32 {}

impl Float for i64 {}

impl Float for i32 {}
