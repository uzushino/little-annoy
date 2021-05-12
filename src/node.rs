
#[derive(Debug, Clone)]
pub struct Node<const N: usize> {
    pub children: Vec<i64>,
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
