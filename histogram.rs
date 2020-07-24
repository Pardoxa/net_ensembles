use serde::{Serialize, Deserialize};

pub trait Histogram {
    /// count val. Some(index), if inside of hist, None if val is invalid
    fn count_index(&mut self, index: usize) -> Result<usize, HistErrors>;
    fn hist(&self) -> &Vec<usize>;
    fn len(&self) -> usize
    {
        self.hist().len()
    }
    /// reset the histogram to zero
    fn reset(&mut self);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistErrors{
    /// A histogram without any bins does not make sense!
    NoBins,

    /// Nothing can hit the bin! (left >= right?)
    IntervalWidthZero,

    /// Invalid value
    OutsideHist

}

pub trait HistogramVal<T>: Histogram{
    fn get_bin_index(&self, val: T) -> Result<usize, HistErrors>;
    /// count val. Some(index), if inside of hist, None if val is invalid
    fn count(&mut self, val: T) -> Result<usize, HistErrors>
    {
        let id = self.get_bin_index(val)?;
        self.count_index(id)
    }
    fn borders(&self) -> Vec<T>;
    fn is_inside(&self, val: T) -> bool;
    fn not_inside(&self, val: T) -> bool;
    fn get_left(&self) -> T;
    fn get_right(&self) -> T;
    fn distance(&self, val: T) -> T;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramGeneric<T>
{
    bin_borders: Vec<T>,
    hist: Vec<usize>,
}

impl<T> Histogram for HistogramGeneric<T>
{
    #[inline]
    fn len(&self) -> usize {
        self.hist.len()
    }

    #[inline]
    fn hist(&self) -> &Vec<usize> {
        &self.hist
    }

    fn count_index(&mut self, index: usize) -> Result<usize, HistErrors> {
        if index < self.len()
        {
            self.hist[index] += 1;
            Ok(index)
        } else {
            Err(HistErrors::OutsideHist)
        }
    }

    fn reset(&mut self) {
        for i in 0..self.len() {
            self.hist[i] = 0;
        }
    }
}

pub type HistogramF64 = HistogramGeneric<f64>;

impl HistogramF64 {
    /// # Create a new Historgram
    /// * right exclusive, left inclusive
    /// * if you want `right` to behave (almost) the same as an inclusive border,
    /// consider using `new(left, right + f64::EPSILON, bins)`
    pub fn new(left: f64, right: f64, bins: usize) -> Result<Self, HistErrors>
    {
        if left >= right {
            return Err(HistErrors::IntervalWidthZero)
        }
        else if bins < 1 {
            return Err(HistErrors::NoBins)
        }

        let bin_size = (right - left) / bins as f64;
        let hist = vec![0; bins];
        let mut bin_borders = Vec::with_capacity(bins + 1);
        bin_borders.extend((0..bins)
            .map(|val| bin_size.mul_add(val as f64, left)) 
        );
        bin_borders.push(right);
        Ok(
            Self{
                bin_borders,
                hist
            }
        )
    }
}

impl HistogramVal<f64> for HistogramF64{

    fn distance(&self, val: f64) -> f64 {
        if self.is_inside(val) {
            0.0
        } else if !val.is_finite() {
            f64::INFINITY
        } else if val < self.get_left() {
            self.get_left() - val
        } else {
            (val - self.get_right()) + f64::EPSILON
        }
    }

    #[inline]
    fn get_left(&self) -> f64 {
        self.bin_borders[0]
    }

    #[inline]
    fn get_right(&self) -> f64 {
        self.bin_borders[self.bin_borders.len() - 1]
    }

    fn is_inside(&self, val: f64) -> bool {
        val >= self.bin_borders[0] 
            && val < self.bin_borders[self.bin_borders.len() - 1]
    }

    fn not_inside(&self, val: f64) -> bool {
        !val.is_finite() 
            || val < self.bin_borders[0] 
            || val >= self.bin_borders[self.bin_borders.len() - 1]
    }


    fn get_bin_index(&self, val: f64) -> Result<usize, HistErrors>
    {
        if self.is_inside(val)
        {
            let search_res = self.bin_borders.binary_search_by(
                |v|
                v.partial_cmp(&val).expect("Should never be NaN")
            );
            match search_res
            {
                Result::Ok(index) => {
                    Ok(index)
                },
                Result::Err(index_p1) => {
                    Ok(index_p1 - 1)
                }
            }
        } else {
            Err(HistErrors::OutsideHist)
        } 
    }

    fn borders(&self) -> Vec<f64> {
        self.bin_borders.clone()
    }
}

pub type HistogramUsize = HistogramGeneric<usize>;

impl HistogramUsize{
    /// right exclusive, left inclusive
    /// (b-a)/bins has to be integer
    pub fn new(left: usize, right: usize, bins: usize) -> Option<Self> {
        assert!(left < right);
        assert!(bins >= 1);
        let bin_size = (right - left) / bins;
        if left + bins * bin_size != right {
            None
        } else {
            let hist = vec![0; bins];
            let bin_borders: Vec<_> = (0..=bins)
                .map(|val| left + val*bin_size)
                .collect();
            Some(
                Self{
                    bin_borders,
                    hist
                }
            )
        }
    }
}

impl HistogramVal<usize> for HistogramUsize{

    fn distance(&self, val: usize) -> usize {
        if self.not_inside(val) {
            if val < self.get_left() {
                self.get_left() - 1
            } else {
                val - self.get_right() + 1
            }
        } else {
            0
        }
    }

    #[inline]
    fn get_left(&self) -> usize {
        self.bin_borders[0]
    }

    #[inline]
    fn get_right(&self) -> usize {
        self.bin_borders[self.bin_borders.len() - 1]
    }

    #[inline]
    fn is_inside(&self, val: usize) -> bool {
        val >= self.get_left()
            && val < self.get_right()
    }

    #[inline]
    fn not_inside(&self, val: usize) -> bool {
        val < self.get_left()
            || val >= self.get_right()
    }

  /// None if not inside Hist covered zone
    fn get_bin_index(&self, val: usize) -> Result<usize, HistErrors>
    {
        if self.not_inside(val)
        {
            return Err(HistErrors::OutsideHist);
        }
        match self.bin_borders.binary_search(&val)
        {
            Result::Ok(index) => {
                Ok(index)
            },
            Result::Err(index_p1) => {
                Ok(index_p1 - 1)
            }
        }
    }

    fn borders(&self) -> Vec<usize> {
        self.bin_borders.clone()
    }
}


/// # Faster version of HistogramUsize
/// provided the bins should be: (left, left +1, ..., right - 1)
/// then you should use this version!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramFast {
    left: usize, 
    right: usize,
    hist: Vec<usize>,
}

impl HistogramFast{
    pub fn new(left: usize, right: usize) -> Result<Self, HistErrors>
    {
        if left >= right {
            Err(HistErrors::OutsideHist)
        } else {
            Ok(
                Self{
                    left,
                    right,
                    hist: vec![0; right - left],
                }
            )
        }
    }

    pub fn new_inclusive(left: usize, right: usize) -> Result<Self, HistErrors>
    {
        Self::new(left, right + 1)
    }
}

impl Histogram for HistogramFast {

    fn count_index(&mut self, index: usize) -> Result<usize, HistErrors> {
        match self.hist.get_mut(index) {
            None => Err(HistErrors::OutsideHist),
            Some(val) => {
                *val += 1;
                Ok(index)
            },
        }
    }

    #[inline]
    fn hist(&self) -> &Vec<usize> {
        &self.hist
    }

    #[inline]
    fn len(&self) -> usize {
        self.hist.len()
    }

    fn reset(&mut self) {
        for i in 0..self.len(){
            self.hist[i] = 0;
        }
    }
}

impl HistogramVal<usize> for HistogramFast
{
    fn get_left(&self) -> usize {
        self.left
    }

    fn get_right(&self) -> usize {
        self.right
    }

    fn distance(&self, val: usize) -> usize {
        if self.not_inside(val) {
            if val < self.get_left() {
                self.get_left() - 1
            } else {
                val - self.get_right() + 1
            }
        } else {
            0
        }
    }

    fn get_bin_index(&self, val: usize) -> Result<usize, HistErrors> {
        if val < self.right{
            match val.checked_sub(self.left) {
                None => Err(HistErrors::OutsideHist),
                Some(index) => Ok(index)
            }
        } else {
            Err(HistErrors::OutsideHist)
        }
    }

    fn borders(&self) -> Vec<usize> {
        (self.left..=self.right).collect()
    }

    fn is_inside(&self, val: usize) -> bool {
        val >= self.left && val < self.right
    }

    fn not_inside(&self, val: usize) -> bool {
        val >= self.right || val < self.left
    }

    fn count(&mut self, val: usize) -> Result<usize, HistErrors> {
        let index = self.get_bin_index(val)?;
        self.hist[index] += 1;
        Ok(index)
    }
}

#[cfg(test)]
mod tests{
    use rand_pcg::Pcg64Mcg;
    use net_ensembles::rand::{distributions::*};
    use super::*;
    #[test]
    fn f64_hist()
    {
        let rng = Pcg64Mcg::new(0xcafef00dd15ea5e5);
        let dist = Uniform::new(f64::EPSILON, 1.0);
        let mut iter = dist.sample_iter(rng);

        for i in 1..100 {
            let left = iter.next().unwrap();
            let right = left + iter.next().unwrap();

            let hist = HistogramF64::new(left, right, i).unwrap();

            assert_eq!(left, hist.get_left(), "i={}", i);
            assert_eq!(right, hist.get_right(), "i={}", i);
            assert_eq!(i+1, hist.borders().len(), "i={}", i);

        }
    }

}