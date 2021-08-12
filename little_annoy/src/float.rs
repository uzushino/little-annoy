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
    + num::traits::real::Real
    + Copy
{
}

impl Float for f64 {}

impl Float for f32 {}
