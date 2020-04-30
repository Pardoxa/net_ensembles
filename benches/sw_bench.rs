use criterion::{criterion_group, criterion_main, Criterion};
use net_ensembles::{SwEnsemble, EmptyNode, traits::*, rand::SeedableRng};
use rand_pcg::Pcg64;
use core::time::Duration;

const SIZE: u32 = 50;

pub fn bench_sw_randomize(c: &mut Criterion) {
    let rng = Pcg64::seed_from_u64(4893);
    let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(SIZE, 0.3, rng);
    c.bench_function("sw_edges_SIZE_random", |b| b.iter(|| e.randomize() ));
}

pub fn bench_sw_markov(c: &mut Criterion) {
    let rng = Pcg64::seed_from_u64(4893);
    let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(SIZE, 0.3, rng);
    c.bench_function("sw_edges_SIZE_m_step", |b| b.iter(|| e.m_steps(10) ));
}

pub fn bench_iterator(c: &mut Criterion) {
    let rng = Pcg64::seed_from_u64(4893);
    let e = SwEnsemble::<EmptyNode, Pcg64>::new(SIZE, 0.3, rng);
    c.bench_function("sw_edges_SIZE_iterator", |b| b.iter(||
        e.container_iter()
        .for_each(
            |con|
            {con.neighbors()
                .count();
            }
        )
    ));
}

pub fn bench_m(c: &mut Criterion) {
    c.bench_function("sw_edges_SIZE_q_core", |b| {
            let rng = Pcg64::seed_from_u64(4893);
            let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(SIZE, 0.3, rng);
            b.iter(||
                {
                    e.m_steps(20);
                    e.graph().q_core(4);
                });

        }

    );
    c.bench_function("sw_edges_SIZE_diameter", |b| {
            let rng = Pcg64::seed_from_u64(4893);
            let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(SIZE, 0.3, rng);
            b.iter(||
                {
                    e.m_steps(20);
                    e.graph().diameter();
                });

        }

    );
    c.bench_function("sw_edges_SIZE_transitivity", |b| {
            let rng = Pcg64::seed_from_u64(4893);
            let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(SIZE, 0.3, rng);
            b.iter(||
                {
                    e.m_steps(20);
                    e.graph().transitivity();
                });

        }

    );
    c.bench_function("sw_edges_SIZE_vertex_load", |b| {
            let rng = Pcg64::seed_from_u64(4893);
            let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(SIZE, 0.3, rng);
            b.iter(||
                {
                    e.m_steps(20);
                    e.graph().vertex_load(true);
                });

        }

    );
    c.bench_function("sw_edges_SIZE_bi_connect", |b| {
            let rng = Pcg64::seed_from_u64(4893);
            let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(SIZE, 0.3, rng);
            b.iter(||
                {
                    e.m_steps(20);
                    e.graph().clone().vertex_biconnected_components(true);
                });

        }

    );
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
    name = sw_bench;
    config = Criterion::default().sample_size(200);
    targets = bench_sw_randomize, bench_sw_markov
}

criterion_main!(measures, benches_iter, sw_bench);
