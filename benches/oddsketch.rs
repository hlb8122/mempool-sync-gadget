#[macro_use]
extern crate criterion;

use criterion::{BatchSize, Criterion};
use mempool_sync::oddsketch::Oddsketch;
use rand::Rng;

fn random_oddsketch() -> Oddsketch {
    let mut rng = rand::thread_rng();
    let mut oddsketch = Oddsketch::default();

    for _ in 0..1024 {
        oddsketch.insert(rng.gen());
    }
    oddsketch
}

fn bench_insert(c: &mut Criterion) {
    c.bench_function("oddsketch insert", move |b| {
        b.iter_batched(
            // Setup
            || (random_oddsketch(), rand::thread_rng().gen()),
            // Loop
            |(mut oddsketch, i)| oddsketch.insert(i),
            BatchSize::SmallInput,
        )
    });
}

fn bench_decode(c: &mut Criterion) {
    c.bench_function("oddsketch decode", move |b| {
        b.iter_batched(
            // Setup
            || (random_oddsketch(), random_oddsketch()),
            // Function
            |(oddsketch_a, oddsketch_b)| (oddsketch_a ^ oddsketch_b).size(),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_insert, bench_decode);
criterion_main!(benches);
