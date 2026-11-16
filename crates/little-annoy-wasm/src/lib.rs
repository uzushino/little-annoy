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

impl Ann {
    fn validate_vector(&self, v: &[f64]) -> Result<(), JsValue> {
        if v.len() != self.ann.dimensions() {
            return Err(JsValue::from_str("vector dimension mismatch"));
        }
        if !v.iter().all(|value| value.is_finite()) {
            return Err(JsValue::from_str("vectors must contain finite numbers"));
        }
        Ok(())
    }
}

#[wasm_bindgen]
impl Ann {
    pub fn new(f: u8) -> Result<Ann, JsValue> {
        if f == 0 {
            return Err(JsValue::from_str("dimensions must be positive"));
        }
        Ok(Ann {
            ann: Annoy::new(usize::from(f)),
        })
    }

    pub fn add_item(&mut self, idx: i32, v: &[f64]) -> Result<(), JsValue> {
        if idx < 0 {
            return Err(JsValue::from_str("item ID must be nonnegative"));
        }
        self.validate_vector(v)?;
        self.ann.add_item(i64::from(idx), v);
        Ok(())
    }

    pub fn build(&mut self, n: i32) -> Result<(), JsValue> {
        if n != -1 && n <= 0 {
            return Err(JsValue::from_str("tree count must be positive or -1"));
        }
        self.ann.build(i64::from(n));
        Ok(())
    }

    pub fn get_nns_by_vector(
        &self,
        v: &[f64],
        n: i32,
        search_k: i32,
    ) -> Result<AnnResult, JsValue> {
        if n < 0 || search_k < -1 {
            return Err(JsValue::from_str(
                "n must be nonnegative; search_k must be nonnegative or -1",
            ));
        }
        self.validate_vector(v)?;
        let (result, distances) = self
            .ann
            .get_nns_by_vector(v, n as usize, i64::from(search_k));
        Ok(AnnResult {
            result: result.into_iter().map(|id| id as i32).collect(),
            dist: distances
                .into_iter()
                .map(|distance| distance as f32)
                .collect(),
        })
    }
}
