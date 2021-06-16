use little_annoy::{Annoy, Euclidean};
use mnist::{Mnist, MnistBuilder};
use rand::Rng;
use rulinalg::matrix::{BaseMatrix, Matrix};

use std::convert::TryInto;

fn vec_to_fixed_slice<T: Copy + Default, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into().unwrap_or_else(|_| [T::default(); N])
}

fn load_mnist(
    size: u32,
    rows: u32,
    cols: u32,
    img: &Vec<u8>,
    lbl: &Vec<u8>,
    index: usize,
) -> (u8, Matrix<u8>) {
    let img = Matrix::new((size * rows) as usize, cols as usize, img.clone());
    let s = index * 28;
    let e = s + 28;
    let row_indexes = (s..e).collect::<Vec<_>>();
    let img = img.select_rows(&row_indexes);

    (lbl[index], img)
}

fn train<const N: usize>(
    ann: &mut Annoy<Euclidean, N>,
    size: u32,
    img: &Vec<u8>,
    lbl: &Vec<u8>,
    rows: u32,
    cols: u32,
) {
    println!("Load mnist data.");

    for i in 0..size {
        let (_, img) = load_mnist(size, rows, cols, &img, &lbl, i as usize);

        let img_to_vec = img
            .data()
            .clone()
            .into_iter()
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        ann.add_item(i as i64, vec_to_fixed_slice(img_to_vec));

        if i % 1_000 == 0 {
            println!("Add item {}/{}.", i, size);
        }
    }

    ann.build(100);
}

fn main() {
    let (trn_size, tst_size, rows, cols) = (100, 100, 28, 28);

    let Mnist {
        trn_img,
        trn_lbl,
        tst_img,
        tst_lbl,
        ..
    } = MnistBuilder::new()
        .label_format_digit()
        .training_set_length(trn_size)
        .validation_set_length(trn_size)
        .test_set_length(tst_size)
        .finalize();

    let mut ann = Annoy::new();
    train(&mut ann, trn_size, &trn_img, &trn_lbl, rows, cols);

    let mut rng = rand::thread_rng();
    for i in 0..10 {
        let ti: u32 = rng.gen();
        let (lbl, img) = load_mnist(
            trn_size,
            rows,
            cols,
            &tst_img,
            &tst_lbl,
            (ti % tst_size) as usize,
        );

        let img_to_vec = img
            .data()
            .clone()
            .into_iter()
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        let arr = vec_to_fixed_slice(img_to_vec) as [f64; 28 * 28];
        let (result, _distance) = ann.get_nns_by_vector(arr, 1, -1);

        let actual = result
            .into_iter()
            .map(|v| trn_lbl[v as usize])
            .collect::<Vec<_>>();

        println!("TEST{}: expected: {}, actual: {:?}", i, lbl, actual);

        if actual[0] != lbl {
            let (_, trn) = load_mnist(100, 28, 28, &trn_img, &trn_lbl, lbl as usize);
            let (_, tst) = load_mnist(100, 28, 28, &tst_img, &tst_lbl, actual[0] as usize);

            println!("{}\n{}", trn, tst);
        }
    }
}
