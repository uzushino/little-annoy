use crate::distance::{normalize, two_means, Distance, NodeImpl};
use crate::random_flip;

use serde::{Deserialize, Serialize};

pub struct Euclidean {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node<const N: usize> {
    pub children: Vec<i64>,
    #[serde(with = "arrays")]
    pub v: [f64; N],
    pub n_descendants: usize,
    pub a: f64,
}

impl<const N: usize> NodeImpl<N> for Node<N> {
    fn new() -> Self {
        Node {
            children: vec![0, 0],
            v: [0.0; N],
            n_descendants: 0,
            a: 0.0,
        }
    }

    fn reset(&mut self, v: [f64; N]) {
        self.children[0] = 0;
        self.children[1] = 0;
        self.n_descendants = 1;
        self.v = v;
    }

    fn descendant(&self) -> usize {
        self.n_descendants
    }

    fn set_descendant(&mut self, other: usize) {
        self.n_descendants = other;
    }

    fn vector(&self) -> [f64; N] {
        self.v
    }

    fn children(&self) -> Vec<i64> {
        self.children.clone()
    }

    fn set_children(&mut self, other: Vec<i64>) {
        self.children = other;
    }

    fn copy(&mut self, other: Self) {
        self.n_descendants = other.n_descendants;
        self.children = other.children;
        self.v = other.v;
        self.a = other.a;
    }
}

impl<const N: usize> Distance<N> for Euclidean {
    type Node = Node<N>;

    fn margin(n: &Self::Node, y: [f64; N]) -> f64 {
        let mut dot: f64 = n.a;

        for z in 0..N {
            dot += n.v[z as usize] * y[z as usize];
        }

        dot
    }

    fn side(n: &Self::Node, y: [f64; N]) -> bool {
        let dot = Self::margin(n, y);

        if dot != 0.0 {
            return dot > 0.0;
        }

        random_flip()
    }

    fn distance(x: [f64; N], y: [f64; N]) -> f64 {
        let mut d = 0.0;
        for i in 0..N {
            d += (x[i as usize] - y[i as usize]) * (x[i as usize] - y[i as usize]);
        }
        d
    }

    fn create_split(nodes: Vec<Self::Node>, n: &mut Self::Node) {
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
