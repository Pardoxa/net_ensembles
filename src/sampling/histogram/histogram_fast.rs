

use num_traits::{int::*, ops::{checked::*, saturating::*, wrapping::*}, cast::*, identities::*, Bounded};
use crate::sampling::histogram::*;
use std::{borrow::*, ops::*};
#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// # Faster version of HistogramInt for Integers
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
    where 
    T: PrimInt + CheckedSub + ToPrimitive + CheckedAdd + One + FromPrimitive
        + HasUnsignedVersion,
    T::Unsigned: Bounded + HasUnsignedVersion<LeBytes=T::LeBytes> 
        + WrappingAdd + ToPrimitive + Sub<Output=T::Unsigned>
{
    /// # Create a new interval
    /// * same as `Self::new_inclusive(left, right - 1)` though with checks
    /// * That makes `left` an inclusive and `right` an exclusive border
    pub fn new(left: T, right: T) -> Result<Self, HistErrors>
    {
        let right = match right.checked_sub(&T::one()){
            Some(res) => res,
            None => return Err(HistErrors::Underflow),
        };
        Self::new_inclusive(left, right)
    }

    /// # Create new histogram with inclusive borders
    /// * `Err` if `left > right`
    /// * `left` is inclusive border
    /// * `right` is inclusive border
    pub fn new_inclusive(left: T, right: T) -> Result<Self, HistErrors>
    {
        if left > right {
            Err(HistErrors::OutsideHist)
        } else {
            let left_u = to_u(left);
            let right_u = to_u(right);
            let size = match (right_u - left_u).to_usize(){
                Some(val) => match val.checked_add(1){
                    Some(val) => val,
                    None => return Err(HistErrors::Overflow),
                },
                None => return Err(HistErrors::UsizeCastError),
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
}


impl<T> HistogramFast<T> 
    where T: PrimInt + CheckedSub + ToPrimitive + CheckedAdd + One + FromPrimitive
        + HasUnsignedVersion + Bounded,
    T::Unsigned: Bounded + HasUnsignedVersion<LeBytes=T::LeBytes, Unsigned=T::Unsigned> 
        + WrappingAdd + ToPrimitive + Sub<Output=T::Unsigned> + FromPrimitive + WrappingSub,
{
    /// 
    pub fn overlapping_partition(&self, n: usize, overlap: usize) -> Vec<Self>
    {
        let mut result = Vec::with_capacity(n);
        let size = self.hist.len() - 1;
        let denominator = n + overlap;
        result.extend(
            (0..n).map(
                |c| 
                {
                    let left_distance = (c * size) / denominator;
                    
                    let left = to_u(self.left) + T::Unsigned::from_usize(left_distance)
                        .unwrap();
                    let right_distance = (c + overlap + 1) * size / denominator;
                    
                    let right = to_u(self.left) + T::Unsigned::from_usize(right_distance)
                        .unwrap();
                  
                    let left = from_u(left);
                    let right = from_u(right);
                    Self::new_inclusive(left, right)
                        .expect("should work?")
                }
            )
        );
        result
    }

}

/// Histogram for binning `usize`- alias for `HistogramFast<usize>`
pub type HistUsizeFast = HistogramFast<usize>;
/// Histogram for binning `u128` - alias for `HistogramFast<u128>`
pub type HistU128Fast = HistogramFast<u128>;
/// Histogram for binning `u64` - alias for `HistogramFast<u64>`
pub type HistU64Fast = HistogramFast<u64>;
/// Histogram for binning `u32` - alias for `HistogramFast<u32>`
pub type HistU32Fast = HistogramFast<u32>;
/// Histogram for binning `u16` - alias for `HistogramFast<u16>`
pub type HistU16Fast = HistogramFast<u16>;
/// Histogram for binning `u8` - alias for `HistogramFast<u8>`
pub type HistU8Fast = HistogramFast<u8>;

/// Histogram for binning `isize` - alias for `HistogramFast<isize>`
pub type HistIsizeFast = HistogramFast<isize>;
/// Histogram for binning `i128` - alias for `HistogramFast<i128>`
pub type HistI128Fast = HistogramFast<i128>;
/// Histogram for binning `i64` - alias for `HistogramFast<i64>`
pub type HistI64Fast = HistogramFast<i64>;
/// Histogram for binning `i32` - alias for `HistogramFast<i32>`
pub type HistI32Fast = HistogramFast<i32>;
/// Histogram for binning `i16` - alias for `HistogramFast<i16>`
pub type HistI16Fast = HistogramFast<i16>;
/// Histogram for binning `i8` - alias for `HistogramFastiu8>`
pub type HistI8Fast = HistogramFast<i8>;


impl<T> Histogram for HistogramFast<T> 
{

    #[inline]
    fn count_index(&mut self, index: usize) -> Result<(), HistErrors> {
        match self.hist.get_mut(index) {
            None => Err(HistErrors::OutsideHist),
            Some(val) => {
                *val += 1;
                Ok(())
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

    #[inline]
    fn reset(&mut self) {
        // compiles to memset =)
        self.hist
            .iter_mut()
            .for_each(|h| *h = 0);
    }
}

impl<T> HistogramVal<T> for HistogramFast<T>
where T: PartialOrd + CheckedSub + CheckedAdd + One + Saturating + NumCast + Copy + std::fmt::Debug,
    std::ops::RangeInclusive<T>: Iterator<Item=T>
{
    #[inline]
    fn get_left(&self) -> T {
        self.left
    }

    #[inline]
    fn get_right(&self) -> T {
        self.right
    }

    fn distance(&self, val: T) -> f64 {
        if self.not_inside(val) {
            let dist = if val < self.get_left() {
                self.get_left() - val
            } else {
                val.saturating_sub(self.right)
            };
            dist.to_f64()
                .unwrap_or(f64::INFINITY)
        } else {
            0.0
        }
    }

    fn get_bin_index<V: Borrow<T>>(&self, val: V) -> Result<usize, HistErrors> {
        let val = *val.borrow();
        if val <= self.right{
            match val.checked_sub(&self.left) {
                None => {
                    let left = self.left.to_isize()
                        .ok_or(HistErrors::CastError)?;
                    let val = val.to_isize()
                        .ok_or(HistErrors::CastError)?;
                    match val.checked_sub(left){
                        None => return Err(HistErrors::OutsideHist),
                        Some(index) => {
                            index.to_usize()
                                .ok_or(HistErrors::OutsideHist)
                        }
                    }
                },
                Some(index) => index.to_usize()
                    .ok_or(HistErrors::OutsideHist)
            }
        } else {
            Err(HistErrors::OutsideHist)
        }
    }

    /// # Creates a vector containing borders (last border is exclusive)
    /// * returns `Err(Overflow)` if right border is `T::MAX`
    /// * creates and returns borders otherwise
    /// * Note: even if `Err(Overflow)` is returned, this does not 
    ///  provide any problems for the rest of the implementation,
    ///  as the border vector is not used internally for `HistogramFast`
    fn borders_clone(&self) -> Result<Vec<T>, HistErrors> {
        let right = self.right.checked_add(&T::one())
            .ok_or(HistErrors::Overflow)?;
        Ok((self.left..=right).collect())
    }

    #[inline]
    fn is_inside<V: Borrow<T>>(&self, val: V) -> bool {
        let val = *val.borrow();
        val >= self.left && val <= self.right
    }

    #[inline]
    fn not_inside<V: Borrow<T>>(&self, val: V) -> bool {
        let val = *val.borrow();
        val > self.right || val < self.left
    }

    #[inline]
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
    fn interval_distance_overlap(&self, val: T, mut overlap: usize) -> usize {
        overlap = overlap.max(1);
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
    use super::*;


    fn hist_test_fast<T>(left: T, right: T)
    where T: PrimInt + num_traits::Bounded + PartialOrd + CheckedSub + One 
        + Saturating + NumCast + Copy + FromPrimitive + std::fmt::Debug + HasUnsignedVersion,
    std::ops::RangeInclusive<T>: Iterator<Item=T>,
    T::Unsigned: Bounded + HasUnsignedVersion<LeBytes=T::LeBytes> 
        + WrappingAdd + ToPrimitive + Sub<Output=T::Unsigned>
    {
        let mut hist = HistogramFast::<T>::new_inclusive(left, right).unwrap();
        assert!(hist.not_inside(T::max_value()));
        assert!(hist.not_inside(T::min_value()));
        for (id, i) in (left..=right).enumerate() {
            assert!(hist.is_inside(i));
            assert_eq!(hist.is_inside(i), !hist.not_inside(i));
            assert!(hist.get_bin_index(i).unwrap() == id);
            assert_eq!(hist.distance(i), 0.0);
            assert_eq!(hist.interval_distance_overlap(i, 2), 0);
            hist.count_val(i).unwrap();
        }
        let lm1 = left - T::one();
        let rp1 = right + T::one();
        assert!(hist.not_inside(lm1));
        assert!(hist.not_inside(rp1));
        assert_eq!(hist.is_inside(lm1), !hist.not_inside(lm1));
        assert_eq!(hist.is_inside(rp1), !hist.not_inside(rp1));
        assert_eq!(hist.distance(lm1), 1.0);
        assert_eq!(hist.distance(rp1), 1.0);
        assert_eq!(hist.interval_distance_overlap(rp1, 1), 1);
        assert_eq!(hist.interval_distance_overlap(lm1, 1), 1);
        assert_eq!(hist.borders_clone().unwrap().len() - 1, hist.bin_count());
    }

    #[test]
    fn hist_fast()
    {
        hist_test_fast(20usize, 31usize);
        hist_test_fast(-23isize, 31isize);
        hist_test_fast(-23i16, 31);
        hist_test_fast(1u8, 3u8);
        hist_test_fast(123u128, 300u128);
        hist_test_fast(-123i128, 300i128);

        hist_test_fast(-100i8, 100i8);
    }

    

    #[test]
    fn hist_creation()
    {
        let _ = HistU8Fast::new_inclusive(0, u8::MAX).unwrap();
        let _ = HistI8Fast::new_inclusive(i8::MIN, i8::MAX).unwrap();
    }

    #[test]
    fn partion_test()
    {
        let h = HistU8Fast::new_inclusive(0, u8::MAX).unwrap();
        let h_part = h.overlapping_partition(2, 0);
        assert_eq!(h.left, h_part[0].left);
        assert_eq!(h.right, h_part.last().unwrap().right);


        let h = HistI8Fast::new_inclusive(i8::MIN, i8::MAX).unwrap();
        let h_part = h.overlapping_partition(2, 0);
        assert_eq!(h.left, h_part[0].left);
        assert_eq!(h.right, h_part.last().unwrap().right);

        let h = HistI16Fast::new_inclusive(i16::MIN, i16::MAX).unwrap();
        let h_part = h.overlapping_partition(2, 2);
        assert_eq!(h.left, h_part[0].left);
        assert_eq!(h.right, h_part.last().unwrap().right);
    }

}