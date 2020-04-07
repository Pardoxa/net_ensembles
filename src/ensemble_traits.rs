
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
    /// * undo a monte carlo step, return result-state
    /// * if you want to undo more than one step
    /// see [`undo_steps`](#method.undo_steps)
    fn undo_step(&mut self, step: S) -> Res;

    /// * undo a monte carlo step, **panic** on invalid result state
    /// * for undoing multiple steps see [`undo_steps_quiet`](#method.undo_steps_quiet)
    fn undo_step_quiet(&mut self, step: S) -> ();

    /// # Randomizes self according to  model
    /// * this is intended for creation of initial sample
    /// * used in [`simple_sample`](#method.simple_sample)
    /// and [`simple_sample_vec`](#method.simple_sample_vec)
    fn randomize(&mut self);

    /// # Monte Carlo step
    /// * use this to perform a Monte Carlo step
    /// * for doing multiple mc steps at once, use [`mc_steps`](#method.mc_steps)
    fn mc_step(&mut self) -> S;

    /// # Monte Carlo steps
    /// * use this to perform multiple Monte Carlo steps at once
    /// * result `Vec<S>` can be used to undo the steps with `self.undo_steps(result)`
    fn mc_steps(&mut self, count: usize) -> Vec<S> {
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(
                self.mc_step()
            );
        }
        vec
    }

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
