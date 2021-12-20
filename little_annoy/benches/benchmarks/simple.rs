use criterion::{criterion_group, Criterion};

use little_annoy::{Annoy, Euclidean};
use rand;

pub fn build(c: &mut Criterion) {
    let mut ann: Annoy<f64, Euclidean> = Annoy::new(2);

    ann.add_item(0, &[1.0, 1.0]);
    ann.add_item(1, &[5.0, 5.0]);
    ann.add_item(2, &[2.0, 2.0]);
    ann.add_item(3, &[4.0, 4.0]);

    for z in 4..10 {
        ann.add_item(z, &[10.0, 10.0]);
    }

    c.bench_function("build 2", |b| b.iter(|| ann.build(2)));
    c.bench_function("build 10", |b| b.iter(|| ann.build(10)));
    c.bench_function("build 100", |b| b.iter(|| ann.build(100)));
}

pub fn add_item(c: &mut Criterion) {
    fn create_item<const N: usize>() -> [f64; N] {
        let mut arr = [0.0; N];
        for i in 0..N {
            arr[i] = rand::random();
        }
        arr
    }

    c.bench_function("add_item 2", |b| {
        b.iter(|| {
            let mut ann: Annoy<f64, Euclidean> = Annoy::new(2);
            for i in 0..100 {
                ann.add_item(i, &create_item::<2>());
            }
            ann.build(100)
        })
    });
    c.bench_function("add_item 100", |b| {
        b.iter(|| {
            let mut ann: Annoy<f64, Euclidean> = Annoy::new(100);
            for i in 0..100 {
                ann.add_item(i, &create_item::<100>());
            }
            ann.build(100)
        })
    });
    c.bench_function("add_item 10_000", |b| {
        b.iter(|| {
            let mut ann: Annoy<f64, Euclidean> = Annoy::new(10_000);
            for i in 0..100 {
                ann.add_item(i, &create_item::<10_000>());
            }
            ann.build(100)
        })
    });
}

criterion_group!(benches, build, add_item);
