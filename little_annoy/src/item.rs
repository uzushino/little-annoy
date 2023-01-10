use num::{
    Zero,
    One,
    ToPrimitive,
    FromPrimitive,
    traits::NumAssign
};

pub trait Item:
    Zero
    + One
    + NumAssign
    + ToPrimitive
    + FromPrimitive
    + PartialOrd
    + Clone
    + Copy
{
    fn sqrt(self) -> Self;
}

impl Item for isize {
    fn sqrt(self) -> Self {
        let v = Self::to_f64(&self)
            .map(|v| v.sqrt())
            .and_then(Self::from_f64);
        v.unwrap_or_else(|| Self::zero())
    }
}

impl Item for i8 {
    fn sqrt(self) -> Self {
        let v = Self::to_f64(&self)
            .map(|v| v.sqrt())
            .and_then(Self::from_f64);
        v.unwrap_or_else(|| Self::zero())
    }
}

impl Item for i16 {
    fn sqrt(self) -> Self {
        let v = Self::to_f64(&self)
            .map(|v| v.sqrt())
            .and_then(Self::from_f64);
        v.unwrap_or_else(|| Self::zero())
    }
}

impl Item for f64 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

impl Item for f32 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

impl Item for i64 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

impl Item for i32 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

impl Item for u64 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

impl Item for u32 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}
