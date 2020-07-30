use num_traits::{int::*, ops::{checked::*, saturating::*}, cast::*, identities::*};
use std::borrow::*;
#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};
/// # Use this has a histogram
/// * anything that implements `Histogram` should also implement the trait `HistogramVal`
pub trait Histogram {
    /// # `self.hist[index] += 1`, `Err()` if `index` out of bounds
    fn count_index(&mut self, index: usize) -> Result<usize, HistErrors>;
    /// # the created histogram
    fn hist(&self) -> &Vec<usize>;
    /// # How many bins the histogram contains
    fn bin_count(&self) -> usize
    {
        self.hist().len()
    }
    /// reset the histogram to zero
    fn reset(&mut self);

    /// check if any bin was not hit yet
    fn any_bin_zero(&self) -> bool
    {
        self.hist()
            .iter()
            .any(|&val| val == 0)
    }
}

/// Possible Errors of the traits `Histogram` and `HistogramVal`
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum HistErrors{
    /// A histogram without any bins does not make sense!
    NoBins,

    /// Nothing can hit the bin! (left >= right?)
    IntervalWidthZero,

    /// Invalid value
    OutsideHist,

    /// Underflow occured
    Underflow,

    /// Overflow occured,
    Overflow,

    /// Error while casting to usize
    UsizeCastError,
}

/// * trait used for mapping values of arbitrary type `T` to bins
/// * used to create a histogram
pub trait HistogramVal<T>: Histogram{
    /// convert val to the respective histogram index
    fn get_bin_index<V: Borrow<T>>(&self, val: V) -> Result<usize, HistErrors>;
    /// count val. Some(index), if inside of hist, None if val is invalid
    fn count_val<V: Borrow<T>>(&mut self, val: V) -> Result<usize, HistErrors>
    {
        let id = self.get_bin_index(val)?;
        self.count_index(id)
    }
    /// # binning borders
    /// * the borders used to bin the values
    /// * any val which fullfills `self.border[i] <= val < self.border[i + 1]` 
    /// will get index `i`.
    /// * **Note** that the last border is exclusive
    fn borders_clone(&self) -> Vec<T>;
    /// does a value correspond to a valid bin?
    fn is_inside<V: Borrow<T>>(&self, val: V) -> bool;
    /// opposite of `is_inside`
    fn not_inside<V: Borrow<T>>(&self, val: V) -> bool;
    /// get the left most border (inclusive)
    fn get_left(&self) -> T;
    /// get the right most border (exclusive)
    fn get_right(&self) -> T;
    /// # calculates some sort of absolute distance to the nearest valid bin
    /// * any invalid numbers (like NAN or INFINITY) should have the highest distance possible
    /// * if a value corresponds to a valid bin, the distance should be zero
    fn distance(&self, val: T) -> f64;
}

/// Distance metric for how far a value is from a valid interval
pub trait HistogramIntervalDistance<T>: HistogramVal<T> {
    /// # Distance metric for how far a value is from a valid interval
    /// * partitions in more intervals, checks which bin interval a bin corresponds to 
    /// and returns distance of said interval to the target interval
    /// * used for heuristiks
    /// * overlap has to be bigger 0
    fn interval_distance_overlap(&self, val: T, overlap: usize) -> usize;
}

/// Generic Histogram struct
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct HistogramGeneric<T>
{
    bin_borders: Vec<T>,
    hist: Vec<usize>,
}

impl<T> HistogramGeneric<T>{
    /// similar to `self.borders_clone` but does not allocate memory
    pub fn borders(&self) -> &Vec<T>
    {
        &self.bin_borders
    }
}

impl<T> Histogram for HistogramGeneric<T>
{
    #[inline]
    fn bin_count(&self) -> usize {
        self.hist.len()
    }

    #[inline]
    fn hist(&self) -> &Vec<usize> {
        &self.hist
    }

    fn count_index(&mut self, index: usize) -> Result<usize, HistErrors> {
        if index < self.bin_count()
        {
            self.hist[index] += 1;
            Ok(index)
        } else {
            Err(HistErrors::OutsideHist)
        }
    }

    fn reset(&mut self) {
        for i in 0..self.bin_count() {
            self.hist[i] = 0;
        }
    }
}

/// Histogram for binning `f64` values
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

    /// Returns the length of the interval
    pub fn interval_length(&self) -> f64
    {
        self.get_right() - self.get_left()
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

    fn is_inside<V: Borrow<f64>>(&self, val: V) -> bool {
        *val.borrow() >= self.bin_borders[0] 
            && *val.borrow() < self.bin_borders[self.bin_borders.len() - 1]
    }

    fn not_inside<V: Borrow<f64>>(&self, val: V) -> bool {
        !(*val.borrow()).is_finite() 
            || *val.borrow() < self.bin_borders[0] 
            || *val.borrow() >= self.bin_borders[self.bin_borders.len() - 1]
    }


    fn get_bin_index<V: Borrow<f64>>(&self, val: V) -> Result<usize, HistErrors>
    {
        let val = val.borrow();
        if self.is_inside(val)
        {
            let search_res = self.bin_borders.binary_search_by(
                |v|
                v.partial_cmp(val).expect("Should never be NaN")
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

    fn borders_clone(&self) -> Vec<f64> {
        self.bin_borders.clone()
    }
}

impl HistogramIntervalDistance<f64> for HistogramF64 {
    fn interval_distance_overlap(&self, val: f64, overlap: usize) -> usize {
        debug_assert!(overlap > 0);
        debug_assert!(self.interval_length() > 0.0);
        debug_assert!(val.is_finite());
        if self.not_inside(val) {
            let num_bins_overlap = self.bin_count() / overlap;
            let dist = 
            if val < self.get_left() { 
                let tmp = self.get_left() - val;
                (tmp / self.interval_length()).floor()
            } else {
                let tmp = val - self.get_right();
                (tmp / self.interval_length()).ceil()
            };
            let int_dist = dist as usize;
            1 + int_dist / num_bins_overlap
        } else {
            0
        }
    }
}

/// # Note: Consider using `HistogramFast` if possible, as it is faster.
/// * `HistogramFast`only works, if every number should correspond to a bin
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

    fn distance(&self, val: usize) -> f64 {
        if self.not_inside(val) {
            let dist = if val < self.get_left() {
                self.get_left() - 1
            } else {
                val - self.get_right() + 1
            };
            dist as f64
        } else {
            0.0
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
    fn is_inside<V: Borrow<usize>>(&self, val: V) -> bool {
        let val = *val.borrow();
        val >= self.get_left()
            && val < self.get_right()
    }

    #[inline]
    fn not_inside<V: Borrow<usize>>(&self, val: V) -> bool {
        let val = *val.borrow();
        val < self.get_left()
            || val >= self.get_right()
    }

    /// None if not inside Hist covered zone
    fn get_bin_index<V: Borrow<usize>>(&self, val: V) -> Result<usize, HistErrors>
    {
        let val = val.borrow();
        if self.not_inside(val)
        {
            return Err(HistErrors::OutsideHist);
        }
        match self.bin_borders.binary_search(val.borrow())
        {
            Result::Ok(index) => {
                Ok(index)
            },
            Result::Err(index_p1) => {
                Ok(index_p1 - 1)
            }
        }
    }

    fn borders_clone(&self) -> Vec<usize> {
        self.bin_borders.clone()
    }
}

impl HistogramIntervalDistance<usize> for HistogramUsize {
    fn interval_distance_overlap(&self, val: usize, overlap: usize) -> usize {
        debug_assert!(overlap > 0);
        if self.not_inside(val) {
            let num_bins_overlap = 1usize.max(self.bin_count() / overlap);
            let dist = 
            if val < self.get_left() { 
                self.get_left() - val
            } else {
                val - self.get_right()
            };
            1 + dist / num_bins_overlap
        } else {
            0
        }
    }
}


/// # Faster version of HistogramGeneric for Integers
/// provided the bins should be: (left, left +1, ..., right - 1)
/// then you should use this version!
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct HistogramFast<T> {
    left: T, 
    right: T,
    hist: Vec<usize>,
}

impl<T> HistogramFast<T> 
    where T: PrimInt + CheckedSub + ToPrimitive + CheckedAdd + One 
{
    /// # Create a new interval
    /// * Err if `left >= right`
    /// * left is inclusive, right is exclusive
    pub fn new(left: T, right: T) -> Result<Self, HistErrors>
    {
        if left >= right {
            Err(HistErrors::OutsideHist)
        } else {
            let size = match right.checked_sub(&left){
                None => return Err(HistErrors::Underflow),
                Some(res) => res
            };
            let size = match size.to_usize() {
                None => return Err(HistErrors::UsizeCastError),
                Some(res) => res,
            };

            Ok(
                Self{
                    left,
                    right,
                    hist: vec![0; size],
                }
            )
        }
    }
    /// same as `self.new`but right is inclusive
    pub fn new_inclusive(left: T, right: T) -> Result<Self, HistErrors>
    {
        let right = match right.checked_add(&T::one()){
            None => return Err(HistErrors::Overflow),
            Some(res) => res,
        };
        Self::new(left, right)
    }
}
/// alias for `HistogramFast<usize>`
pub type HistogramFastUsize = HistogramFast<usize>;
/// alias for `HistogramFast<u64>`
pub type HistogramFastU64 = HistogramFast<u64>;
/// alias for `HistogramFast<u32>`
pub type HistogramFastU32 = HistogramFast<u32>;
/// alias for `HistogramFast<u16>`
pub type HistogramFastU16 = HistogramFast<u16>;
/// alias for `HistogramFast<u8>`
pub type HistogramFastU8 = HistogramFast<u8>;

/// alias for `HistogramFast<isize>`
pub type HistogramFastIsize = HistogramFast<isize>;
/// alias for `HistogramFast<i64>`
pub type HistogramFastI64 = HistogramFast<i64>;
/// alias for `HistogramFast<i32>`
pub type HistogramFastI32 = HistogramFast<i32>;
/// alias for `HistogramFast<i16>`
pub type HistogramFastI16 = HistogramFast<i16>;
/// alias for `HistogramFastiu8>`
pub type HistogramFastI8 = HistogramFast<i8>;


impl<T> Histogram for HistogramFast<T> 
{

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
    fn bin_count(&self) -> usize {
        self.hist.len()
    }

    fn reset(&mut self) {
        for i in 0..self.bin_count(){
            self.hist[i] = 0;
        }
    }
}

impl<T> HistogramVal<T> for HistogramFast<T>
where T: PartialOrd + CheckedSub + One + Saturating + NumCast + Copy,
    std::ops::RangeInclusive<T>: Iterator<Item=T>
{
    fn get_left(&self) -> T {
        self.left
    }

    fn get_right(&self) -> T {
        self.right
    }

    fn distance(&self, val: T) -> f64 {
        if self.not_inside(val) {
            let dist = if val < self.get_left() {
                self.get_left() - val
            } else {
                val.saturating_sub(self.right)
                    .saturating_add(T::one())
            };
            dist.to_f64()
                .unwrap_or(f64::INFINITY)
        } else {
            0.0
        }
    }

    fn get_bin_index<V: Borrow<T>>(&self, val: V) -> Result<usize, HistErrors> {
        let val = *val.borrow();
        if val < self.right{
            match val.checked_sub(&self.left) {
                None => Err(HistErrors::OutsideHist),
                Some(index) => Ok(index.to_usize().unwrap())
            }
        } else {
            Err(HistErrors::OutsideHist)
        }
    }

    fn borders_clone(&self) -> Vec<T> {
        (self.left..=self.right).collect()
    }

    fn is_inside<V: Borrow<T>>(&self, val: V) -> bool {
        let val = *val.borrow();
        val >= self.left && val < self.right
    }

    fn not_inside<V: Borrow<T>>(&self, val: V) -> bool {
        let val = *val.borrow();
        val >= self.right || val < self.left
    }

    fn count_val<V: Borrow<T>>(&mut self, val: V) -> Result<usize, HistErrors> {
        let index = self.get_bin_index(val)?;
        self.hist[index] += 1;
        Ok(index)
    }
}

impl<T> HistogramIntervalDistance<T> for HistogramFast<T> 
where Self: HistogramVal<T>,
    T: PartialOrd + std::ops::Sub<Output=T> + NumCast + Copy
{
    fn interval_distance_overlap(&self, val: T, overlap: usize) -> usize {
        debug_assert!(overlap > 0);
        if self.not_inside(val) {
            let num_bins_overlap = 1usize.max(self.bin_count() / overlap);
            let dist = 
            if val < self.left { 
                self.left - val
            } else {
                val - self.right
            };
            1 + dist.to_usize().unwrap() / num_bins_overlap
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests{
    use rand_pcg::Pcg64Mcg;
    use crate::rand::{distributions::*};
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
    fn hist_test<T>(left: T, right: T)
    where T: PrimInt + num_traits::Bounded + PartialOrd + CheckedSub + One + Saturating + NumCast + Copy,
    std::ops::RangeInclusive<T>: Iterator<Item=T>
    {
        let hist = HistogramFast::<T>::new_inclusive(left, right).unwrap();
        assert!(hist.not_inside(T::max_value()));
        assert!(hist.not_inside(T::min_value()));
        for (id, i) in (left..=right).enumerate() {
            assert!(hist.is_inside(i));
            assert!(hist.get_bin_index(i).unwrap() == id);
            assert_eq!(hist.distance(i), 0.0);
            assert_eq!(hist.interval_distance_overlap(i, 2), 0);
        }
        assert!(hist.not_inside(left - T::one()));
        assert!(hist.not_inside(right + T::one()));
    }

    #[test]
    fn hist_fast()
    {
        hist_test(20usize, 31usize);
        hist_test(-23isize, 31isize);
    }

}