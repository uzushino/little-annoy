use wasm_bindgen::prelude::*;
use little_annoy::{Annoy, Euclidean};

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Mutex;

use std::sync::RwLock;
#[macro_use]
use lazy_static::lazy_static;

lazy_static! {
    static ref ANN: Mutex<Annoy<f64, Euclidean, 2>> = {
        let mut ann = Annoy::new();
        Mutex::new(ann)
    };
}


#[wasm_bindgen]
pub fn eucridian() -> Result<(), JsValue> {
    let _ = ANN.lock().and_then(|mut ann| {
        ann.add_item(0, [1.0, 1.0]);
        ann.add_item(1, [5.0, 5.0]);
        ann.add_item(2, [2.0, 2.0]);
        ann.add_item(3, [4.0, 4.0]);
        ann.build(100);

        let (result, distance) = ann.get_nns_by_vector([1.0, 1.0], 10, -1);

        for (i, id) in result.iter().enumerate() {
            println!("result = {}, distance = {}", *id, distance[i]);
        }

        Ok(ann)
    }).unwrap();

    Ok(())
}
