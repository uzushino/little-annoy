use criterion::{criterion_group, Criterion};

use little_annoy::Annoy;

pub fn build(c: &mut Criterion) {
    let mut ann = Annoy::new();
    
    ann.add_item(0, [1.0, 1.0]);
    ann.add_item(1, [5.0, 5.0]);
    ann.add_item(2, [2.0, 2.0]);
    ann.add_item(3, [4.0, 4.0]);

    for z in 4..10 {
        ann.add_item(z, [10.0, 10.0]);
    }

    c.bench_function("build 2", |b| b.iter(|| ann.build(20)));
}

criterion_group!(benches, build);
