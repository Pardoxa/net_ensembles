use std::io::Write;

/// # Traits for quantities that all Entropic simulations have
pub trait Entropic
{
    /// # Counter
    /// how many Entropic steps were performed until now?
    fn step_counter(&self) -> usize;

    /// # Counter
    /// * how many markov steps were performed until now?
    /// * this includes steps, that were perfored to find the estimate, i.e., during wang landau
    fn steps_total(&self) -> usize{
        self.total_steps_accepted() + self.total_steps_rejected()
    }

    /// # How many steps were accepted until now?
    /// * this includes steps, that were perfored to find the estimate, i.e., during wang landau
    fn total_steps_accepted(&self) -> usize;

    /// # How many steps were rejected until now?
    /// * this includes steps, that were perfored to find the estimate, i.e., during wang landau
    fn total_steps_rejected(&self) -> usize;

    /// # Calculate, which fraction of steps were accepted
    /// * this includes steps, that were perfored to find the estimate, i.e., during wang landau
    /// * if no steps were performed, it returns `f64::NAN`
    fn fraction_accepted_total(&self) -> f64 {
        let total_acc = self.total_steps_accepted();
        let total_steps = total_acc + self.total_steps_rejected();

        if total_steps == 0 {
            f64::NAN
        } else {
            total_acc as f64 / total_steps as f64
        }
    }

    /// # Calculate, which fraction of steps were rejected
    /// * this includes steps, that were perfored to find the estimate, i.e., during wang landau
    /// * if no steps were performed, it returns `f64::NAN`
    fn fraction_rejected_total(&self) -> f64 {
        let total_rej = self.total_steps_rejected();
        let total_steps = total_rej + self.total_steps_accepted();

        if total_steps == 0 {
            f64::NAN
        } else {
            total_rej as f64 / total_steps as f64
        }
    }

    /// # What is the goal to reach?
    fn step_goal(&self) -> usize;
    
    /// # Checks wang landau threshold
    /// * `log_f <= log_f_threshold`
    fn is_finished(&self) -> bool{
        self.step_counter() >= self.step_goal()
    }
    
    /// # Current (non normalized) estimate of ln(P(E))
    /// * i.e., of the natural logarithm of the 
    /// probability density function
    /// for the requested interval
    /// * this is what we are doing the simulations for
    fn log_density(&self) -> Vec<f64>;

    /// # Current (non normalized) estimate of log10(P(E))
    /// * i.e., of logarithm with base 10 of the 
    /// probability density function
    /// for the requested interval
    /// * this is what we are doing the simulations for
    fn log_density_base10(&self) -> Vec<f64>{
        let mut density = self.log_density();
        density
            .iter_mut()
            .for_each(|val| *val *= std::f64::consts::LOG10_E);
        density
        
    }

    /// # Current (non normalized) estimate of log_base(P(E))
    /// * i.e., of logarithm with arbitrary base of the 
    /// probability density function
    /// for the requested interval
    /// * this is what we are doing the simulations for
    fn log_density_base(&self, base: f64) -> Vec<f64>{
        let factor = std::f64::consts::E.log(base);
        let mut density = self.log_density();
        density
            .iter_mut()
            .for_each(|val| *val *= factor);
        density
    }

    /// Writes Information about the simulation to a file.
    /// E.g. How many steps were performed.
    fn write_log<W: Write>(&self, writer: W) -> Result<(), std::io::Error>;
    
}

/// # trait to request a reference to the current (state of the) ensemble 
/// * See also [EntropicEEH](trait.EntropicEEH.html)
pub trait EntropicEnsemble<E> : Entropic
{
    /// return reference to current state of ensemble
    fn ensemble(&self) -> &E;

    /// # returns mutable reference to ensemble
    /// * If, whatever you do with the ensemble, changes the 
    ///   energy of the current state, 
    /// you cannot trust the results of entropic sampling anymore
    /// * use with care
    unsafe fn ensemble_mut(&mut self) -> &mut E;
}

/// # trait to request the current histogram from a Entropic simulation
/// * Note: The histogram will likely be reset multiple times during a simulation
/// * See also [EntropicEEH](trait.EntropicEEH.html)
pub trait EntropicHist<Hist> : Entropic
{
    /// # returns current histogram
    /// * **Note**: histogram will be reset multiple times during the simulation
    /// * please refere to the [papers](struct.EntropicAdaptive.html#adaptive-Entropic-1t)
    fn hist(&self) -> &Hist;
}

/// # trait to request the current energy from a Entropic simulation
/// * `None` if the energy was not calculated yet
/// * See also [EntropicEEH](trait.EntropicEEH.html)
pub trait EntropicEnergy<Energy> : Entropic
{
    /// returns the last accepted `Energy` calculated
    /// `None` if no energy was calculated yet
    fn energy(&self) -> &Energy;
}

/// Helper trait, so that you have to type less
pub trait EntropicEEH<E, Hist, Energy> 
    : EntropicEnergy<Energy> + EntropicEnsemble<E>
        + EntropicHist<Hist>{}

impl<A, E, Hist, Energy> EntropicEEH<E, Hist, Energy> for A
    where 
    A: EntropicEnergy<Energy> 
        + EntropicEnsemble<E>
        + EntropicHist<Hist>{}