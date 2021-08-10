pub trait Float :
    num::Num +
    num::Zero +
    num::traits::NumAssign +
    num::traits::Signed +
    num::Float +
    num::traits::FloatConst +
    num::ToPrimitive +
    num::FromPrimitive +
    PartialEq +
    PartialOrd +
    Clone +
    Copy {
        fn one() -> Self;
}

impl Float for f64 {
    fn one() -> Self { 1.0 }
}

impl Float for f32 {
    fn one() -> Self { 1.0 }
}
