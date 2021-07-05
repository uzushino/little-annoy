use wasm_bindgen::prelude::*;
use little_annoy::{Annoy, Euclidean};

use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref ANN: Mutex<Annoy<f64, Euclidean, 2>> = {
        let mut ann = Annoy::new();
        Mutex::new(ann)
    };
}

#[wasm_bindgen]
pub fn build() -> Result<(), JsValue> {
    let _ = ANN.lock().and_then(|mut ann| {
        ann.add_item(0, [1.0, 1.0]);
        ann.add_item(1, [5.0, 5.0]);
        ann.add_item(2, [2.0, 2.0]);
        ann.add_item(3, [4.0, 4.0]);

        ann.build(100);

        Ok(ann)
    }).unwrap();

    Ok(())
}

#[wasm_bindgen]
pub fn get_nns_by_vector() -> Result<(), JsValue> {
    let result = ANN.lock().and_then(|mut ann| {
        let (result, distance) = ann.get_nns_by_vector([1.0, 1.0], 10, -1);
        Ok((result, distance))
    }).unwrap();

    Ok(())
}
