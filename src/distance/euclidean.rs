use crate::distance::{ two_means, normalize, Distance};
use crate::random_flip;

use serde::{Serialize, Deserialize};

pub struct Euclidean {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node<const N: usize> {
    pub children: Vec<i64>,
    #[serde(with = "arrays")]
    pub v: [f64; N],
    pub n_descendants: usize,
    pub a: f64,
}

impl<const N: usize> Node<N> {
    pub fn new() -> Self {
        Node {
            children: vec![0, 0],
            v: [0.0; N],
            n_descendants: 0,
            a: 0.0,
        }
    }
}

impl Euclidean {
    fn margin<const N: usize>(n: &Node<N>, y: [f64; N]) -> f64 {
        let mut dot: f64 = n.a;

        for z in 0..N {
            dot += n.v[z as usize] * y[z as usize];
        }

        dot
    }
}

impl Distance for Euclidean {
    fn side<const N: usize>(n: &Node<N>, y: [f64; N]) -> bool {
        let dot = Self::margin(n, y);
        if dot != 0.0 {
            return dot > 0.0;
        }
        random_flip()
    }

    fn distance<const N: usize>(x: [f64; N], y: [f64; N]) -> f64 {
        let mut d = 0.0;
        for i in 0..N {
            d += ((x[i as usize] - y[i as usize])) * ((x[i as usize] - y[i as usize]));
        }
        d
    }

    fn create_split<const N: usize>(nodes: Vec<Node<N>>, n: &mut Node<N>) {
        let (best_iv, best_jv) = two_means::<Euclidean, N>(nodes);

        for z in 0..N {
            n.v[z] = best_iv[z] - best_jv[z];
        }

        normalize(&mut n.v);

        n.a = 0.0;
        
        for z in 0..N {
            n.a += -n.v[z] * (best_iv[z] + best_jv[z]) / 2.0;
        }
    }
}

// https://github.com/serde-rs/serde/issues/1937#issuecomment-812137971
mod arrays {
    use std::{convert::TryInto, marker::PhantomData};

    use serde::{
        de::{SeqAccess, Visitor},
        ser::SerializeTuple,
        Deserialize, Deserializer, Serialize, Serializer,
    };
    pub fn serialize<S: Serializer, T: Serialize, const N: usize>(
        data: &[T; N],
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = ser.serialize_tuple(N)?;
        for item in data {
            s.serialize_element(item)?;
        }
        s.end()
    }

    struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

    impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
    where
        T: Deserialize<'de>,
    {
        type Value = [T; N];

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(&format!("an array of length {}", N))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // can be optimized using MaybeUninit
            let mut data = Vec::with_capacity(N);
            for _ in 0..N {
                match (seq.next_element())? {
                    Some(val) => data.push(val),
                    None => return Err(serde::de::Error::invalid_length(N, &self)),
                }
            }
            match data.try_into() {
                Ok(arr) => Ok(arr),
                Err(_) => unreachable!(),
            }
        }
    }
    pub fn deserialize<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        deserializer.deserialize_tuple(N, ArrayVisitor::<T, N>(PhantomData))
    }
}
