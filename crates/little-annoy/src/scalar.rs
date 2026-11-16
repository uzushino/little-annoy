use num_traits::{FromPrimitive, NumAssign, Signed, ToPrimitive};
use serde::{Serialize, de::DeserializeOwned};

pub trait Scalar:
    'static
    + Send
    + Sync
    + Serialize
    + DeserializeOwned
    + NumAssign
    + ToPrimitive
    + FromPrimitive
    + PartialOrd
    + Signed
    + Copy
{
    fn sqrt(self) -> Self {
        let v = Self::to_f64(&self)
            .map(|v| v.sqrt())
            .and_then(Self::from_f64);
        v.unwrap_or_else(|| Self::zero())
    }
}

impl Scalar for isize {}

impl Scalar for i8 {}

impl Scalar for i16 {}

impl Scalar for f64 {}

impl Scalar for f32 {}

impl Scalar for i64 {}

impl Scalar for i32 {}
