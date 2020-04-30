use criterion::{criterion_group, criterion_main, Criterion};
use net_ensembles::{SwEnsemble, EmptyNode, rand::SeedableRng};
use rand_pcg::Pcg64;
use core::time::Duration;
mod common;
use common::*;

const SIZE: u32 = 50;
const SEED: u64 = 123_239_010;

fn init() -> SwEnsemble<EmptyNode, Pcg64> {
    let rng = Pcg64::seed_from_u64(SEED);
    SwEnsemble::<EmptyNode, Pcg64>::new(SIZE, 0.3, rng)
}

pub fn sw_steps_bench(c: &mut Criterion) {
    generic_steps_bench(c, "sw", init);
}

pub fn bench_iterator(c: &mut Criterion) {
    generic_iter_bench(c, "sw", init);
}

pub fn bench_m(c: &mut Criterion) {
    generic_measure_bench(c, "sw", init);
}



criterion_group!{
    name = measures;
    config = Criterion::default()
        .sample_size(200)
        .warm_up_time( Duration::new(1, 1))
        .measurement_time(Duration::new(30, 0));
    targets = bench_m
}

criterion_group!{
    name = benches_iter;
    config = Criterion::default().sample_size(200);
    targets = bench_iterator
}

criterion_group!{
    name = sw_steps;
    config = Criterion::default().sample_size(200);
    targets = sw_steps_bench
}

criterion_main!(measures, benches_iter, sw_steps);
