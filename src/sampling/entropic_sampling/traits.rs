use std::io::Write;

/// # Traits for quantities that all Entropic simulations have
pub trait Entropic
{
    /// # Counter
    /// how many wang Landau steps were performed until now?
    fn step_counter(&self) -> usize;

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
        let factor = std::f64::consts::E.log10();
        let mut density = self.log_density();
        density
            .iter_mut()
            .for_each(|val| *val *= factor);
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
