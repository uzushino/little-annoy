use little_annoy::{Annoy, Euclidean};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Ann {
    ann: Annoy<f64, Euclidean>,
}

#[wasm_bindgen]
pub struct AnnResult {
    result: Box<[i32]>,
    dist: Box<[f32]>,
}

#[wasm_bindgen]
impl AnnResult {
    pub fn result(&self) -> Box<[i32]> {
        self.result.clone()
    }

    pub fn distance(&self) -> Box<[f32]> {
        self.dist.clone()
    }
}

#[wasm_bindgen]
impl Ann {
    pub fn new(f: u8) -> Ann {
        let ann: Annoy<f64, Euclidean> = Annoy::new(f as usize);
        Ann { ann }
    }

    pub fn add_item(&mut self, idx: i32, v: &[f64]) {
        self.ann.add_item(idx as i64, v);
    }

    pub fn build(&mut self, n: i32) {
        self.ann.build(n as i64);
    }

    pub fn get_nns_by_vector(&mut self, v: &[f64], n: i32, search_k: i32) -> AnnResult {
        let (r, d) = self.ann.get_nns_by_vector(v, n as usize, search_k as i64);

        AnnResult {
            result: r
                .into_iter()
                .map(|n| n as i32)
                .collect::<Vec<i32>>()
                .into_boxed_slice(),
            dist: d
                .into_iter()
                .map(|n| n as f32)
                .collect::<Vec<f32>>()
                .into_boxed_slice(),
        }
    }
}
