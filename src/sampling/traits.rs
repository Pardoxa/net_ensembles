
use crate::sampling::MetropolisState;

/// # Create a markov chain by doing markov steps
pub trait MarkovChain<S, Res> {
    /// * undo a markov step, return result-state
    /// * if you want to undo more than one step
    /// see [`undo_steps`](#method.undo_steps)
    fn undo_step(&mut self, step: S) -> Res;

    /// * undo a markov, **panic** on invalid result state
    /// * for undoing multiple steps see [`undo_steps_quiet`](#method.undo_steps_quiet)
    fn undo_step_quiet(&mut self, step: S);

    /// # Markov step
    /// * use this to perform a markov step step
    /// * for doing multiple markov steps at once, use [`m_steps`](#method.m_steps)
    fn m_step(&mut self) -> S;

    /// #  Markov steps
    /// * use this to perform multiple markov steps at once
    /// * result `Vec<S>` can be used to undo the steps with `self.undo_steps(result)`
    fn m_steps(&mut self, count: usize) -> Vec<S> {
        let mut vec = Vec::with_capacity(count);
        vec.extend((0..count)
            .map(|_| self.m_step())
        );
        vec
    }

    /// # Markov steps without return
    /// * use this to perform multiple markov steps at once
    /// * only use this if you **know** that you do **not** want to undo the steps
    /// * you cannot undo this steps, but therefore it does not need to allocate a vector 
    /// for undoing steps
    fn m_steps_quiet(&mut self, count: usize)
    {
        for _ in 0..count {
            self.m_step();
        }
    }

    /// # Undo markov steps
    /// * Note: uses undo_step in correct order and returns result
    /// ## Important:
    /// * look at specific implementation of `undo_step`, every thing mentioned there applies to each step
    fn undo_steps(&mut self, steps: Vec<S>) -> Vec<Res> {
        steps.into_iter()
            .rev()
            .map(|step| self.undo_step(step))
            .collect()
    }

    /// # Undo markov steps
    /// * Note: uses `undo_step_quiet` in correct order
    /// ## Important:
    /// * look at specific implementation of `undo_step_quiet`, every thing mentioned there applies to each step
    fn undo_steps_quiet(&mut self, steps: Vec<S>) {
        let iter = steps.into_iter()
            .rev();
        for step in iter {
            self.undo_step_quiet(step);
        }
    }

}

/// Use the metropolis algorithm
pub trait Metropolis<S, Res>: MarkovChain<S, Res> {
    /// # Metropolis algorithm
    /// **panics**: `stepsize = 0` is not allowed and will result in a panic
    ///
    /// |               | meaning                                                                      |
    /// |---------------|------------------------------------------------------------------------------|
    /// | `rng`         | the Rng used to decide, if a state should be accepted or rejected            |
    /// | `temperature` | used in metropolis probability                                               |
    /// | `stepsize`    | is used for each markov step, i.e., `self.m_steps(stepsize)` is called       |
    /// | `steps`       | is the number of steps with size `stepsize`, that this method should perform |
    /// | `valid_self`  | checks, if the markov steps produced a valid state                           |
    /// | `energy`      | should calculate the "energy" of the system used for acceptance probability  |
    /// | `measure`     | called after each step                                                       |
    ///
    /// **Important**: if possible, treat `energy(&mut Self)` as `energy(&Self)`. This will be safer.
    ///
    /// **Note**: instead of the `temperature` T the literature sometimes uses &beta;. The relation between them is:
    /// &beta; = T⁻¹
    ///
    /// **Note**: If `valid_self` returns `false`, the state will be rejected. If you do not need this,
    /// use `|_| true`
    ///
    /// **`measure`**: function is intended for storing measurable quantities etc.
    /// Is called at the end of each iteration. As for the parameter:
    ///
    /// | type        | name suggestion  | description                                                                                                                                  |
    /// |-------------|------------------|----------------------------------------------------------------------------------------------------------------------------------------------|
    /// | `&mut Self` | `current_state`  | current `self`. After stepping and accepting/rejecting                                                                                       |
    /// | `usize`     | `i`              | counter, starts at 0, each step counter increments by 1                                                                                      |
    /// | `f64`       | `current_energy` | `energy(&mut self)`. Assumes that `energy` of a state deterministic and invariant under: `let steps = self.steps(n); self.undo_steps(steps);`|
    /// | `bool`      | `rejected`       | `true` if last step was rejected. That should mean, that the current state is the same as the last state.                                    |
    ///
    /// Citation see, e.g,
    /// > M. E. J. Newman and G. T. Barkema, "Monte Carlo Methods in Statistical Physics"
    ///   *Clarendon Press*, 1999, ISBN:&nbsp;978-0-19-8517979
    ///
    /// # Explanation
    /// * Performes markov chain using the markov chain trait
    ///
    /// Let the current state of the system be S(i) with corresponding energy `E(i) = energy(S(i))`.
    /// Now perform a markov step, such that the new system is Snew with energy Enew.
    /// The new state will be accepted (meaning S(i+1) = Snew) with probability:
    /// `min[1.0, exp{-1/T * (Enew - E(i))}]`
    /// otherwise the new state will be rejected, meaning S(i + 1) = S(i).
    /// Afterwards, `measure` is called.
    #[allow(clippy::clippy::too_many_arguments)]
    fn metropolis<Rng, F, G, H>(
        &mut self,
        rng: Rng,
        temperature: f64,
        stepsize: usize,
        steps: usize,
        valid_self: F,
        energy: G,
        measure: H,
    ) -> MetropolisState<Rng>
    where
        F: FnMut(&mut Self) -> bool,
        G: FnMut(&mut Self) -> f64,
        H: FnMut(&mut Self, usize, f64, bool),
        Rng: rand::Rng
    {
        self.metropolis_while(
            rng,
            temperature,
            stepsize,
            steps,
            valid_self,
            energy,
            measure,
            |_, _| false
        )
    }

    /// same as `metropolis`, but checks function `break_if(current_state, counter)` after each step and
    /// stops if `true` is returned.
    #[allow(clippy::clippy::too_many_arguments)]
    fn metropolis_while<Rng, F, G, H, B>(
        &mut self,
        mut rng: Rng,
        temperature: f64,
        stepsize: usize,
        steps: usize,
        mut valid_self: F,
        mut energy: G,
        mut measure: H,
        mut brake_if: B,
    ) -> MetropolisState<Rng>
    where
        F: FnMut(&mut Self) -> bool,
        G: FnMut(&mut Self) -> f64,
        H: FnMut(&mut Self, usize, f64, bool),
        B: FnMut(&Self, usize) -> bool,
        Rng: rand::Rng
    {
        assert!(
            stepsize > 0,
            "StepSize 0 is not allowed!"
        );
        let mut old_energy = energy(self);
        let mut current_energy = old_energy;
        let mut last_steps: Vec<_>;
        let mut a_prob: f64;
        let m_beta = -1.0 / temperature;

        for i in 0..steps {
            last_steps = self.m_steps(stepsize);

            // calculate acceptance probability
            let mut rejected = !valid_self(self);
            if !rejected {
                current_energy = energy(self);
                // I only have to calculate this for a valid state
                a_prob = 1.0_f64.min((m_beta * (current_energy - old_energy)).exp());
                rejected = rng.gen::<f64>() > a_prob;
            }


            // if step is NOT accepted
            if rejected {
                self.undo_steps_quiet(last_steps);
                current_energy = old_energy;
            } else {
                old_energy = current_energy;
            }
            measure(self, i, current_energy, rejected);

            if brake_if(self, i) {
                #[cold]
                return MetropolisState::new(stepsize, steps, m_beta, rng, current_energy, i + 1);
            }
        }

        MetropolisState::new(stepsize, steps, m_beta, rng, current_energy, steps)
    }

    /// # resume the metropolis
    /// continues metropolis_while from a specific state.
    ///
    /// Note: this is intended to be used after loading a savestate, e.g., from a file.
    /// You have to store both the MetropolisState and the ensemble
    ///
    /// * asserts, that the `energy(self)` matches the energy stored in `state`. Can be turned of
    /// with `ignore_energy_missmatch = true`
    ///
    /// # Example:
    /// ```
    /// use net_ensembles::sampling::{MetropolisSave, MetropolisState};
    /// use net_ensembles::sampling::traits::{Metropolis, SimpleSample};
    /// use net_ensembles::{ErEnsembleC, EmptyNode};
    /// use net_ensembles::traits::MeasurableGraphQuantities;
    /// // as an alternative to the above, you can use the import from the next line
    /// // use net_ensembles::{*, sampling::*};
    ///
    /// use net_ensembles::rand::SeedableRng; // reexported
    /// use rand_pcg::Pcg64;
    /// use serde_json;
    /// use std::fs::File;
    ///
    /// // first init an ensemble, which implements MarkovChain
    /// let rng = Pcg64::seed_from_u64(7567526);
    /// let mut ensemble = ErEnsembleC::<EmptyNode, _>::new(300, 4.0, rng);
    ///
    /// // ensure that inital state is valid
    /// while !ensemble.is_connected().unwrap() {
    ///     ensemble.randomize();
    /// }
    ///
    /// // now perform metropolis
    /// // in this example the simulation will be interrupted, when the counter hits 20:
    /// // break_if = |_, counter| counter == 20
    /// let metropolis_rng = Pcg64::seed_from_u64(77526);
    /// let state = ensemble.metropolis_while(
    ///     metropolis_rng, // rng
    ///     -10.0,          // temperature
    ///     30,             // stepsize
    ///     100,            // steps
    ///     |ensemble| ensemble.is_connected().unwrap(),    // valid_self
    ///     |ensemble| ensemble.diameter().unwrap() as f64, // energy
    ///     |ensemble, counter, energy, rejected| {         // measure
    ///         // of cause, you can store it in a file instead
    ///         println!("{}, {}, {}, {}", counter, rejected, energy, ensemble.leaf_count());
    ///     },
    ///     |_, counter| counter == 20,                     // break_if
    /// );
    ///
    /// // NOTE: You will likely not need the cfg part
    /// // I only need it, because the example has to work with and without serde_support
    /// #[cfg(feature = "serde_support")]
    /// {
    ///     // saving
    ///     let save_file = File::create("metropolis.save")
    ///         .expect("Unable to create file");
    ///
    ///     let save = MetropolisSave::new(ensemble, state);
    ///     serde_json::to_writer_pretty(save_file, &save)
    ///         .unwrap();
    ///
    ///
    ///     // loading
    ///     let reader = File::open("metropolis.save")
    ///         .expect("Unable to open file");
    ///
    ///     let save: MetropolisSave::<ErEnsembleC::<EmptyNode, Pcg64>, Pcg64>
    ///         = serde_json::from_reader(reader).unwrap();
    ///
    ///     let (mut loaded_ensemble, loaded_state) = save.unpack();
    ///
    ///     // resume the simulation
    ///     loaded_ensemble.continue_metropolis_while(
    ///         loaded_state,
    ///         false,                   // asserting that current energy equals stored energy
    ///         |ensemble| ensemble.is_connected().unwrap(),    // valid_self
    ///         |ensemble| ensemble.diameter().unwrap() as f64, // energy
    ///         |ensemble, counter, energy, rejected| {         // measure
    ///             // of cause, you could store it in a file instead
    ///             // and you could use the `rejected` variable to only calculate other
    ///             // quantities, if something actually changed
    ///             println!("{}, {}, {}, {}", counter, rejected, energy, ensemble.leaf_count());
    ///         },
    ///         |_, _| false,                     // break_if
    ///     );
    /// }
    /// ```
    #[allow(clippy::clippy::too_many_arguments, clippy::float_cmp)]
    fn continue_metropolis_while<R, F, G, H, B>(
        &mut self,
        state: MetropolisState<R>,
        ignore_energy_missmatch: bool,
        mut valid_self: F,
        mut energy: G,
        mut measure: H,
        mut brake_if: B,
    ) -> MetropolisState<R>
    where
        F: FnMut(&mut Self) -> bool,
        G: FnMut(&mut Self) -> f64,
        H: FnMut(&mut Self, usize, f64, bool),
        B: FnMut(&Self, usize) -> bool,
        R: rand::Rng
    {
        let mut old_energy = energy(self);
        if !ignore_energy_missmatch {
            assert_eq!(
                old_energy,
                state.current_energy(),
                "Energy missmatch!"
            );
        }
        let mut current_energy = old_energy;
        let mut last_steps: Vec<_>;
        let mut a_prob: f64;
        let m_beta = state.m_beta();
        let steps = state.step_target();
        let stepsize = state.stepsize();
        let counter = state.counter();
        let mut rng: R = state.into_rng();

        for i in counter..steps {
            last_steps = self.m_steps(stepsize);

            // calculate acceptance probability
            let mut rejected = !valid_self(self);
            if !rejected {
                // I only have to calculate this for a valid state
                current_energy = energy(self);

                a_prob = 1.0_f64.min((m_beta * (current_energy - old_energy)).exp());
                rejected = rng.gen::<f64>() > a_prob;
            }


            // if step is NOT accepted
            if rejected {
                self.undo_steps_quiet(last_steps);
                current_energy = old_energy;
            } else {
                old_energy = current_energy;
            }
            measure(self, i, current_energy, rejected);

            if brake_if(self, i) {
                #[cold]
                return MetropolisState::new(stepsize, steps, m_beta, rng, current_energy, i + 1);
            }
        }

        MetropolisState::new(stepsize, steps, m_beta, rng, current_energy, steps)
    }


    /// same as `continue_metropolis_while`, but without the `brake_if`
    fn continue_metropolis<Rng, F, G, H>(
        &mut self,
        state: MetropolisState<Rng>,
        ignore_energy_missmatch: bool,
        valid_self: F,
        energy: G,
        measure: H,
    ) -> MetropolisState<Rng>
    where
        F: FnMut(&mut Self) -> bool,
        G: FnMut(&mut Self) -> f64,
        H: FnMut(&mut Self, usize, f64, bool),
        Rng: rand::Rng
    {
        self.continue_metropolis_while(
            state,
            ignore_energy_missmatch,
            valid_self,
            energy,
            measure,
            |_, _| false
        )
    }
}

impl<S, Res, A> Metropolis<S, Res> for A
where A: MarkovChain<S, Res> { }

/// For easy sampling of your ensemble
pub trait SimpleSample{
    /// # Randomizes self according to  model
    /// * this is intended for creation of initial sample
    /// * used in [`simple_sample`](#method.simple_sample)
    /// and [`simple_sample_vec`](#method.simple_sample_vec)
    fn randomize(&mut self);

    /// # do the following `times` times:
    /// 1) `f(self)`
    /// 2) `self.randomize()`
    fn simple_sample<F>(&mut self, times: usize, mut f: F)
        where F: FnMut(&Self) -> ()
    {
        for _ in 0..times {
            f(self);
            self.randomize();
        }
    }

    /// # do the following `times` times:
    /// 1) `res = f(self)`
    /// 2) `self.randomize()`
    /// ## res is collected into Vector
    fn simple_sample_vec<F, G>(&mut self, times: usize, mut f: F) -> Vec<G>
        where F: FnMut(&Self) -> G
    {
        let mut vec = Vec::with_capacity(times);
        for _ in 0..times {
            vec.push(f(self));
            self.randomize();
        }
        vec
    }
}


/// # Access internal random number generator
pub trait HasRng<Rng>
where Rng: rand::Rng
{
    /// # Access RNG
    /// If, for some reason, you want access to the internal random number generator: Here you go
    fn rng(&mut self) -> &mut Rng;

    /// # If you need to exchange the internal rng
    /// * swaps internal random number generator with `rng`
    fn swap_rng(&mut self, rng: &mut Rng);
}