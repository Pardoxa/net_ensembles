use net_ensembles::*;
use criterion::{Criterion};


pub fn generic_steps_bench<'a, T, A, E, F, M1, M2> (c: &mut Criterion, name: &str, mut init: F)
where
    T: Node,
    A: AdjContainer<T>,
    E: WithGraph<T, GenericGraph<T, A>> + SimpleSample + MarkovChain<M1, M2>,
    F: FnMut() -> E
{
    let mut e = init();
    let rand_name = format!("randomize_{}", name);
    c.bench_function(&rand_name, |b| b.iter(|| e.randomize() ));

    e = init();
    let m_step_name = format!("m_steps_10_{}", name);
    c.bench_function(&m_step_name, |b| b.iter(|| e.m_steps(10) ));
}

pub fn generic_measure_bench<'a, T, A, E, F, M1, M2> (c: &mut Criterion, name: &str, mut init: F)
where
    T: Node,
    A: AdjContainer<T>,
    E: WithGraph<T, GenericGraph<T, A>> + MarkovChain<M1, M2>,
    GenericGraph<T, A>: Clone,
    F: FnMut() -> E
{
    let name_q_core = format!("measure_4_core_{}", name);
    c.bench_function(&name_q_core, |b| {
            let mut e = init();
            b.iter(||
                {
                    e.m_steps(20);
                    e.graph().q_core(4);
                });

        }

    );

    let name_diameter = format!("measure_diameter_{}", name);
    c.bench_function(&name_diameter, |b| {
            let mut e = init();
            b.iter(||
                {
                    e.m_steps(20);
                    e.graph().diameter();
                });

        }

    );

    let name_transitivity = format!("measure_transitivity_{}", name);
    c.bench_function(&name_transitivity, |b| {
            let mut e = init();
            b.iter(||
                {
                    e.m_steps(20);
                    e.graph().transitivity();
                });

        }

    );

    let name_vertex_load = format!("measure_vertex_load_{}", name);
    c.bench_function(&name_vertex_load, |b| {
            let mut e = init();
            b.iter(||
                {
                    e.m_steps(20);
                    e.graph().vertex_load(true);
                });

        }

    );

    let name_bi_connect = format!("measure_bi_connect_{}", name);

    c.bench_function(&name_bi_connect, |b| {
            let mut e = init();
            b.iter(||
                {
                    e.m_steps(20);
                    e.graph().clone().vertex_biconnected_components(true);
                });

        }

    );
}

pub fn generic_iter_bench<'a, T, A, E, F> (c: &mut Criterion, name: &str, mut init: F)
where
    T: Node,
    A: AdjContainer<T>,
    E: WithGraph<T, GenericGraph<T, A>> + SimpleSample,
    GenericGraph<T, A>: Clone,
    F: FnMut() -> E
{

    let mut e = init();
    let iter_name = format!("iter_bench_{}", name);
    c.bench_function(&iter_name, |b|
    {
        e.randomize();
        b.iter(||
            e.container_iter()
            .for_each(
                |con|
                {con.neighbors()
                    .count();
                }
            )
        );
    });

}
