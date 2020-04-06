
/// # Access random number generator of Ensemble
pub trait EnsembleRng<A, B, Rng>
where Self: Ensemble<A,B>,
      Rng: rand::Rng
{
    /// # Access RNG
    /// If, for some reason, you want access to the internal random number generator: Here you go
    fn rng(&mut self) -> &mut Rng;

    /// # If you need to exchange the internal rng
    /// * returns old rng
    fn swap_rng(&mut self, rng: Rng) -> Rng;
}

/// # Sampling an Ensemble
/// ## includes:
/// * monte carlo steps
/// * simple sampling
pub trait Ensemble<S, Res> {
    /// undo a monte carlo step, return result-state
    fn undo_step(&mut self, step: S) -> Res;

    /// undo a monte carlo step, **panic** on invalid result state
    fn undo_step_quiet(&mut self, step: S) -> ();

    /// # Randomizes self according to  model
    /// * this is intended for creation of initial sample
    /// * you can use this for sampling the ensemble
    fn randomize(&mut self);

    /// # Monte Carlo step
    /// * use this to perform a Monte Carlo step
    fn mc_step(&mut self) -> S;

    /// # Monte Carlo steps
    /// * use this to perform multiple Monte Carlo steps at once
    /// * result `Vec<S>` can be used to undo the steps with `self.undo_steps(result)`
    fn mc_steps(&mut self, count: usize) -> Vec<S> {
        (0..count)
            .map(|_| self.mc_step())
            .collect()
    }

    /// # do the following `times` times:
    /// 1) `self.randomize();`
    /// 2) `f(self)`
    fn simple_sample<F>(&mut self, times: usize, f: F)
        where F: Fn(&Self) -> ()
    {
        for _ in 0..times {
            self.randomize();
            f(self);
        }
    }

    /// # do the following times` times:
    /// 1) `self.randomize()`
    /// 2) `res = f(self)`
    /// ## res is collected into Vector
    fn simple_sample_vec<F, G>(&mut self, times: usize, f: F) -> Vec<G>
        where F: Fn(&Self) -> G
    {
        (0..times).map(
            |_|
            {
                self.randomize();
                f(self)
            }
        ).collect()

    }

    /// # Undo Monte Carlo steps
    /// * Note: uses undo_step in correct order and returns result
    /// ## Important:
    /// * look at specific implementation of `undo_step`, every thing mentioned there applies to each step
    fn undo_steps(&mut self, steps: Vec<S>) -> Vec<Res> {
        steps.into_iter()
            .rev()
            .map(|step| self.undo_step(step))
            .collect()
    }

    /// # Undo Monte Carlo steps
    /// * Note: uses `undo_step_quiet` in correct order
    /// ## Important:
    /// * look at specific implementation of `undo_step_quiet`, every thing mentioned there applies to each step
    fn undo_steps_quiet(&mut self, steps: Vec<S>) -> () {
        let iter = steps.into_iter()
            .rev();
        for step in iter {
            self.undo_step_quiet(step);
        }
    }
}
