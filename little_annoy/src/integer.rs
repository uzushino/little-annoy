
pub trait Integer :
    PartialEq +
    PartialOrd +
    Clone +
    Copy {
        fn one() -> Self;
}

impl Integer for i64 {
    fn one() -> Self { 1 }
}

impl Integer for i32 {
    fn one() -> Self { 1 }
}
