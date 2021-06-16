use little_annoy::{Annoy, Hamming};

use bit_vec::BitVec;
use img_hash::HasherConfig;
use std::convert::TryInto;

fn vec_to_fixed_slice<T: Copy + Default, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into().unwrap_or_else(|v: Vec<T>| [T::default(); N])
}

pub fn to_binary(elem: u8) -> Vec<bool> {
    let byte_vec = elem.to_le_bytes().to_vec();
    let bv = BitVec::from_bytes(&byte_vec);
    return bv.iter().collect::<Vec<bool>>();
}

fn main() {
    let mut ann: Annoy<Hamming, 64> = Annoy::new();

    let image1 = image::open("data/a.jpg").unwrap();
    let image2 = image::open("data/b.jpg").unwrap();

    let hasher = HasherConfig::new().to_hasher();

    let hash1 = hasher.hash_image(&image1);
    let hash2 = hasher.hash_image(&image2);

    let image1_binary: Vec<u8> = hash1
        .as_bytes()
        .iter()
        .map(|x| to_binary(*x))
        .flatten()
        .map(|x| x as u8)
        .collect();
    let image2_binary: Vec<u8> = hash2
        .as_bytes()
        .iter()
        .map(|x| to_binary(*x))
        .flatten()
        .map(|x| x as u8)
        .collect();
    let image1_binary: [f64; 64] = vec_to_fixed_slice(image1_binary) as [f64; 64];
    let image2_binary: [f64; 64] = vec_to_fixed_slice(image2_binary) as [f64; 64];

    ann.add_item(1, image1_binary);
    ann.add_item(2, image2_binary);
    ann.add_item(3, image2_binary);
    ann.add_item(4, image2_binary);

    ann.build(100);

    let (result, distance) = ann.get_nns_by_vector(image1_binary, 1, -1);

    println!("result: {:?}, distance: {:?}", result, distance)
}
