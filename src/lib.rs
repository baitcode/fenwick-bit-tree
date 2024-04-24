//! # Slighly over-engineered Fenwick Tree implmentation.
//! 
//! Allows efficient prefix sum calculation.
//! 
//! Created for training purposes to test: 
//!
//! 1. rust typesystem, default trait implmentation, enums as a way for polymorphism
//! 2. memory management and consumption of value
//! 3. cargo tools, docs, tests, clippy and benchmarks, build and publish.
//!
//! Code is free to do whatever you feel like.
//! 
//! Provides abstraction for Fenwick tree data structure and 2 implmentations:
//!  - [`prelude::FixedSizeFenwickTree`]
//!  - [`prelude::GrowingFenwickTree`]
//! 
//! Key space for a tree lies within [`usize`] range. Tree support any value that 
//! implements [`FenwickTreeValue`] trait. [`FenwickTreeValue`] is automatically 
//! implmented for all primitive numeric types that support [`std::ops::AddAssign`], 
//! [`std::ops::Sub`], [`core::cmp::PartialEq`] and [`Copy`] traits.
//!
//! ## Installation  
//!
//! ```bash
//! cargo install fenwick_bit_tree
//! ```
//! 
//! ## Test
//! 
//! ```bash
//! cargo test
//! ```
//! 
//! ## Benchmarks
//! 
//! ```bash
//! cargo bench --features benchmarks
//! ```
//! 
//! ## Basic usage:
//! 
//! ```rust
//! use fenwick_bit_tree::prelude::*;
//! 
//! // Create the tree with capacity for 32 aggregated [`i32`] data points. 
//! // One can use whole usize range to store datapoints for unicode timestamps
//! let mut tree = FixedSizeFenwickTree::<i32>::new(32);
//!
//! // Add values
//! 
//! tree.update(0, 1); 
//! tree.update(0, 4); // Will aggregate value at index 0 so it would be 5
//! tree.update(10, 10);
//! tree.update(20, 10);
//! tree.update(30, 10);
//! 
//! // Now you can query data. 
//! // NOTE: FixedSizeFenwickTree will raise error when query goes out of bounds.
//! //       GrowingFenwickTree will automatically truncate the range to the rightmost index. 
//! 
//! assert_eq!(tree.query(4).unwrap(), 5); 
//! assert_eq!(tree.query(15).unwrap(), 15);
//! assert_eq!(tree.query(31).unwrap(), 35);
//!
//! // Also allows making range queries
//! 
//! let val = tree.range_query(2, 16).unwrap(); // Will return aggregated sum of all values between those keys.
//! assert_eq!(val, 10);
//! ```

#![forbid(unsafe_code)]
#![feature(test)]

use std::ops::{Deref, DerefMut};

mod fixed_size_tree;
mod growing_tree;

pub use fixed_size_tree::FixedSizeFenwickTree;
pub use growing_tree::GrowingFenwickTree;

/// Contains all public types
pub mod prelude {
    pub use crate::FenwickTreeValue;
    pub use crate::fixed_size_tree::FixedSizeFenwickTree;
    pub use crate::growing_tree::GrowingFenwickTree;
    pub use crate::FenwickTree;
    pub use crate::TreeError;
}

fn least_significant_bit(idx: usize) -> usize {
    let int_idx = idx as i32;
    (int_idx & -int_idx) as usize
}

/// Types that implement that trait can be stored and aggregated within Fenwick tree.
pub trait FenwickTreeValue:
    Default + Clone //
    + core::cmp::PartialEq 
{
    fn store_value(&mut self, other: &Self);
    fn substract(self, other: Self) -> Self;
}

impl<T> FenwickTreeValue for T 
where T: Default + Copy //
    + std::ops::AddAssign
    + std::ops::Sub<Output = Self>
    + core::cmp::PartialEq 
{
    fn store_value(&mut self, other: &Self) {
        *self += *other
    }

    fn substract(self, other: Self) -> Self {
        self - other
    }
}

/// Fenwick tree trait, API of that data structure
pub trait FenwickTree {
    type Value: FenwickTreeValue;

    /// Returns sum of values across all indexes lesser or equal than `idx`.
    ///
    /// # Errors
    ///
    /// This function will returns an error if idx is out of bounds.
    /// GrowingFenwick tree implementation never returns error.
    /// 
    fn query(&self, idx: usize) -> Result<Self::Value, TreeError>;
    
    /// Add new value to the `idx` stored value, which is 0 by default. 
    ///
    /// # Errors
    ///
    /// This function will return an error if idx is out of bounds.
    /// GrowingFenwick tree implementation never returns error.
    /// 
    fn update(&mut self, idx: usize, value: Self::Value) -> Result<(), TreeError>;

    /// Returns sum of values across all indexes in between `from` and `to` indexes 
    /// (including edges).
    ///
    /// # Errors
    ///
    /// This function will return an error if any index is out of bounds.
    /// GrowingFenwick tree implementation never return error.
    /// 
    fn range_query(&self, from: usize, to: usize) -> Result<Self::Value, TreeError> {
        let from_sum = self.query(from)?;
        let to_sum = self.query(to)?;
        Ok(to_sum.substract(from_sum))
    }
}

/// For the sake of clarity Tree supports 2 types of indexing. [`TreeIndex::External`] is meant to be used 
/// by library consumer. While [`TreeIndex::Internal`] is used for purposes to make tree reindexing code more
/// understable and maintainable. [`usize`] can be automatically converted using `into()` into the [`TreeIndex::External`]
#[derive(Debug, Clone, Copy)]
enum TreeIndex {
    Internal { val: usize },
    External { val: usize },
}

#[derive(Debug, PartialEq)]
pub enum TreeError {
    IndexOutOfBounds( usize )
}

impl TreeIndex {

    fn to_internal(self) -> Self {
        match self {
            TreeIndex::Internal { val: _ } => self,
            TreeIndex::External { val } => TreeIndex::Internal { val: val + 1 },
        }
    }

    fn to_external(self) -> Result<Self, String> {
        match self {
            TreeIndex::Internal { val } => {
                if val == 0 {
                    return Err("Index is out of bounds.".to_string());
                }
                Ok(TreeIndex::External { val: val - 1 })
            }
            TreeIndex::External { val: _ } => Ok(self),
        }
    }

    /// Starts with the initial value and then moves down to zero returning result of
    /// deduction of the least significant bit
    fn lsb_descending(self) -> LeastSignificantBitDescentingChain {
        LeastSignificantBitDescentingChain {
            idx: self.to_internal(),
        }
    }

    /// Starts with the initial value and then moves up until upper bound is reached 
    /// returning the result of deduction of the least significant bit
    fn lsb_ascending(self, upper_bound: usize) -> LeastSignificantBitAscendingChain {
        LeastSignificantBitAscendingChain {
            idx: self.to_internal(),
            max: upper_bound,
        }
    }

    fn is_power_of_2(self) -> bool {
        let idx = *self;
        idx.is_power_of_two()
    }

}

impl From<usize> for TreeIndex {
    fn from(value: usize) -> Self {
        Self::External { val: value }
    }
}

impl Deref for TreeIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        match self {
            TreeIndex::External { val } => val,
            TreeIndex::Internal { val } => val,
        }
    }
}

impl PartialEq for TreeIndex {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Internal { val: l_val }, Self::Internal { val: r_val }) => l_val == r_val,
            (Self::External { val: l_val }, Self::External { val: r_val }) => l_val == r_val,
            _ => false,
        }
    }
}

impl DerefMut for TreeIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            TreeIndex::External { val } => val,
            TreeIndex::Internal { val } => val,
        }
    }
}

/// Iterator that implements changing value by deduction of the least significant bit and 
/// returning result
struct LeastSignificantBitDescentingChain {
    idx: TreeIndex,
}

impl Iterator for LeastSignificantBitDescentingChain {
    type Item = TreeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if *self.idx == 0 {
            return None;
        }
        // TODO: implement COpy?
        let res = TreeIndex::Internal { val: *self.idx };
        *self.idx -= least_significant_bit(*self.idx);
        Some(res)
    }
}

/// Iterator that implements changing value by addition of the least significant bit and 
/// returning result
struct LeastSignificantBitAscendingChain {
    idx: TreeIndex,
    max: usize,
}

impl Iterator for LeastSignificantBitAscendingChain {
    type Item = TreeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if *self.idx > self.max {
            return None;
        }
        // TODO: implement COpy?
        let res = TreeIndex::Internal { val: *self.idx };
        *self.idx += least_significant_bit(*self.idx);
        Some(res)
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;

    use crate::{least_significant_bit, TreeIndex};

    fn to_internal_index_vec(indexes: &[usize]) -> Vec<TreeIndex> {
        indexes
            .into_iter()
            .map(|i| TreeIndex::Internal { val: *i })
            .collect::<Vec<TreeIndex>>()
    }

    #[test]
    fn test_index_transform_from_internal_to_external_with_error() {
        let idx = TreeIndex::Internal { val: 0 };
        idx.to_external().expect_err("Index is out of bounds.");
    }

    #[test]
    fn test_index_transform_from_internal_to_external() {
        for val in 1..100 {
            let idx = TreeIndex::Internal { val: val };
            assert_eq!(
                idx.to_external().unwrap(),
                TreeIndex::External { val: val - 1 }
            );
        }
    }

    #[test]
    fn test_index_transform_from_external_to_internal() {
        for val in 0..100 {
            let idx = TreeIndex::External { val: val };
            assert_eq!(idx.to_internal(), TreeIndex::Internal { val: val + 1 });
        }
    }

    #[test]
    fn test_index_transform_to_itseld() {
        for val in 0..100 {
            let idx = TreeIndex::External { val: val };
            assert_eq!(idx.to_external().unwrap(), TreeIndex::External { val });
        }

        for val in 0..100 {
            let idx = TreeIndex::Internal { val: val };
            assert_eq!(idx.to_internal(), TreeIndex::Internal { val: val });
        }
    }

    #[test]
    fn test_ascending_lsb_chain() {
        let idx: TreeIndex = 0.into();
        assert_eq!(
            idx.lsb_ascending(64).collect::<Vec<TreeIndex>>(),
            to_internal_index_vec(&[1, 2, 4, 8, 16, 32, 64])
        );

        let idx: TreeIndex = 1.into();
        assert_eq!(
            idx.lsb_ascending(64).collect::<Vec<TreeIndex>>(),
            to_internal_index_vec(&[2, 4, 8, 16, 32, 64])
        );

        let idx: TreeIndex = 6.into();
        assert_eq!(
            idx.lsb_ascending(64).collect::<Vec<TreeIndex>>(),
            to_internal_index_vec(&[7, 8, 16, 32, 64])
        );

        let idx: TreeIndex = 6.into();
        assert_eq!(idx.lsb_ascending(0).collect::<Vec<TreeIndex>>(), vec![]);
    }

    #[test]
    fn test_descending_lsb_chain() {
        let idx: TreeIndex = 5.into();
        assert_eq!(idx, TreeIndex::External { val: 5 });
        assert_eq!(
            idx.lsb_descending().collect::<Vec<TreeIndex>>(),
            to_internal_index_vec(&[6, 4])
        );

        let idx: TreeIndex = 4.into();
        assert_eq!(
            idx.lsb_descending().collect::<Vec<TreeIndex>>(),
            to_internal_index_vec(&[5, 4])
        );

        let idx = TreeIndex::Internal { val: 3 };
        assert_eq!(
            idx.lsb_descending().collect::<Vec<TreeIndex>>(),
            to_internal_index_vec(&[3, 2])
        );

        let idx = TreeIndex::Internal { val: 12 };
        assert_eq!(
            idx.lsb_descending().collect::<Vec<TreeIndex>>(),
            to_internal_index_vec(&[12, 8])
        );
    }

    #[test]
    fn test_lsb() {
        assert_eq!(least_significant_bit(12), 4)
    }

    #[test]
    fn test_bitwise_op() {
        assert_eq!(12usize.next_power_of_two(), 16);
        assert_eq!(12usize.next_power_of_two() >> 1, 8);
    }
}
