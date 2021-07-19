use little_annoy::{Annoy, Euclidean};
use wasm_bindgen::prelude::*;
use web_sys::console::*;

#[wasm_bindgen]
pub fn ann_new(f: u8) -> *mut Annoy<f64, Euclidean> {
    let ann: Annoy<f64, Euclidean>  = Annoy::new(f as usize);
    &ann as *const _ as *mut _
}

#[wasm_bindgen]
pub fn build(ann_ptr: *mut Annoy<f64, Euclidean>, n: i64) -> Result<(), JsValue>{
    let ann = unsafe { ann_ptr.as_mut() };
    
    if let Some(ann) = ann {
        ann.build(n);
    }

    Ok(())
}

#[wasm_bindgen]
pub fn add_item(ann_ptr: *mut Annoy<f64, Euclidean>, idx: u32, v: &[f64]) -> Result<(), JsValue>{
    let ann = unsafe { ann_ptr.as_mut() };
    if let Some(ann) = ann {
        ann.add_item(idx as i64, v);
    }

    Ok(())
}
