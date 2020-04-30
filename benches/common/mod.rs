use net_ensembles::*;
use criterion::{Criterion, black_box};


pub fn generic_steps_bench<'a, T, A, E, F, M1, M2> (c: &mut Criterion, name: &str, step_size: usize, mut init: F)
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
    let m_step_name = format!("m_steps_{}_{}", step_size, name);
    c.bench_function(&m_step_name, |b| b.iter(|| e.m_steps(step_size) ));
}

pub fn generic_measure_bench<'a, T, A, E, F, M1, M2> (c: &mut Criterion, name: &str, step_size: usize, mut init: F)
where
    T: Node,
    A: AdjContainer<T>,
    E: WithGraph<T, GenericGraph<T, A>> + MarkovChain<M1, M2>,
    GenericGraph<T, A>: Clone,
    F: FnMut() -> E
{
    let name_q_core = format!("measure_{}_m_step_{}_4_core", name, step_size);
    c.bench_function(&name_q_core, |b| {
            let mut e = init();
            b.iter(||
                {
                    e.m_steps(step_size);
                    e.graph().q_core(4);
                });

        }

    );

    let name_diameter = format!("measure_{}_m_step_{}_diameter", name, step_size);
    c.bench_function(&name_diameter, |b| {
            let mut e = init();
            b.iter(||
                {
                    e.m_steps(step_size);
                    e.graph().diameter();
                });

        }

    );

    let name_transitivity = format!("measure_{}_m_step_{}_transitivity", name, step_size);
    c.bench_function(&name_transitivity, |b| {
            let mut e = init();
            b.iter(||
                {
                    e.m_steps(step_size);
                    e.graph().transitivity();
                });

        }

    );

    let name_vertex_load = format!("measure_{}_m_step_{}_vertex_load", name, step_size);
    c.bench_function(&name_vertex_load, |b| {
            let mut e = init();
            b.iter(||
                {
                    e.m_steps(step_size);
                    e.graph().vertex_load(true);
                });

        }

    );

    let name_bi_connect = format!("measure_{}_m_step_{}_bi_connect", name, step_size);

    c.bench_function(&name_bi_connect, |b| {
            let mut e = init();
            b.iter(||
                {
                    e.m_steps(step_size);
                    e.graph().clone().vertex_biconnected_components(true);
                });

        }

    );
}


pub fn generic_simple_measure_bench<'a, T, A, E, F> (c: &mut Criterion, name: &str, mut init: F)
where
    T: Node,
    A: AdjContainer<T>,
    E: WithGraph<T, GenericGraph<T, A>> + SimpleSample,
    GenericGraph<T, A>: Clone,
    F: FnMut() -> E
{
    let name_q_core = format!("s_measure_{}_4_core", name);
    let mut e = init();
    c.bench_function(&name_q_core, |b| {
        e.randomize();
            b.iter(||
                {
                    e.graph().q_core(4);
                });

        }

    );

    let name_diameter = format!("s_measure_{}_diameter", name);
    let mut e = init();
    c.bench_function(&name_diameter, |b| {
        e.randomize();
            b.iter(||
                {
                    e.graph().diameter();
                });

        }

    );

    let name_transitivity = format!("s_measure_{}_transitivity", name);
    let mut e = init();
    c.bench_function(&name_transitivity, |b| {
            e.randomize();
            b.iter(||
                {
                    e.graph().transitivity();
                });

        }

    );

    let name_vertex_load = format!("s_measure_{}_vertex_load", name);
    let mut e = init();
    c.bench_function(&name_vertex_load, |b| {
            e.randomize();
            b.iter(||
                {
                    e.graph().vertex_load(true);
                });

        }

    );

    let name_bi_connect = format!("s_measure_{}_bi_connect", name);

    let mut e = init();
    c.bench_function(&name_bi_connect, |b| {
            e.randomize();
            b.iter(||
                {
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
                {
                    let iter = con.neighbors();
                    black_box(iter.count());
                }
            )
        );
    });

}
