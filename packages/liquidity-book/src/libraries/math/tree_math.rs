//! ### Liquidity Book Tree Math Library
//! Author: Kent and Haseeb
//!
//! This module contains functions to interact with a tree of TreeUint24.

use super::{bit_math::BitMath, u24::U24};
use crate::Bytes32;
use ethnum::U256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: This module is likely inefficient because we don't have bit ops for Bytes32, so we have to
// convert into U256 a lot. Other parts of the library could also benefit from Bytes32 bit ops.

pub trait TreeUint24 {
    fn contains(&self, id: u32) -> bool;
    fn add(&mut self, id: u32) -> bool;
    fn remove(&mut self, id: u32) -> bool;
    fn find_first_right(&self, id: u32) -> u32;
    fn find_first_left(&self, id: u32) -> u32;
    fn _closest_bit_right(leaves: U256, bit: u8) -> U256;
    fn _closest_bit_left(leaves: U256, bit: u8) -> U256;
}

/// Can store 256^3 = 2^24 = 16,777,216 values.
/// Each bit represents whether a bin is non-empty (1) or empty (0).
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MockTreeUint24 {
    pub level0: Bytes32,                   // 256 possible values
    pub level1: HashMap<Bytes32, Bytes32>, // 256^2 possible values
    pub level2: HashMap<Bytes32, Bytes32>, // 256^3 possible values
}

impl MockTreeUint24 {
    /// Creates a new empty TreeUint24.
    pub fn new() -> Self {
        MockTreeUint24 {
            level0: Bytes32::default(),
            level1: HashMap::<Bytes32, Bytes32>::new(),
            level2: HashMap::<Bytes32, Bytes32>::new(),
        }
    }
}

impl TreeUint24 for MockTreeUint24 {
    /// Checks if the tree contains the given `id`.
    ///
    /// Returns `true` if the tree contains the `id`.
    fn contains(&self, id: u32) -> bool {
        let key2 = U256::from(id) >> 8u8;

        U256::from_le_bytes(*self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]))
            & (U256::from(1u32) << (id & 255u32))
            != U256::ZERO
    }

    /// Adds the given `id` to the tree.
    ///
    /// Returns `true` if the `id` was not already in the tree.
    /// If the `id` was already in the tree, no changes are made and `false` is returned.
    fn add(&mut self, id: u32) -> bool {
        let key2 = U256::from(id) >> 8u8;

        let leaves =
            U256::from_le_bytes(*self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]));
        let new_leaves = leaves | U256::ONE << (id & u8::MAX as u32);

        if leaves != new_leaves {
            self.level2
                .insert(key2.to_le_bytes(), new_leaves.to_le_bytes());

            if leaves == U256::ZERO {
                let key1 = key2 >> 8u8;
                let leaves = U256::from_le_bytes(
                    *self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                let value1 = leaves | (U256::ONE << (key2 & U256::from(u8::MAX)));

                self.level1.insert(key1.to_le_bytes(), value1.to_le_bytes());

                if leaves == U256::ZERO {
                    let value0 = U256::from_le_bytes(self.level0)
                        | (U256::ONE << (key1 & U256::from(u8::MAX)));
                    self.level0 = value0.to_le_bytes();
                }
            }
            return true;
        }

        false
    }

    /// Removes the given `id` from the tree.
    ///
    /// Returns `true` if the `id` was in the tree.
    /// If the `id` was not in the tree, no changes are made and `false` is returned.
    fn remove(&mut self, id: u32) -> bool {
        let key2 = U256::from(id) >> 8u8;

        let leaves =
            U256::from_le_bytes(*self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]));
        let new_leaves = leaves & !(U256::ONE << (id & u8::MAX as u32));

        if leaves != new_leaves {
            self.level2
                .insert(key2.to_le_bytes(), new_leaves.to_le_bytes());

            if new_leaves == U256::ZERO {
                let key1 = key2 >> 8u8;
                let leaves = U256::from_le_bytes(
                    *self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                let value1 = leaves & !(U256::ONE << (key2 & U256::from(u8::MAX)));
                self.level1.insert(key1.to_le_bytes(), value1.to_le_bytes());

                if leaves == U256::ZERO {
                    let value0 = U256::from_le_bytes(self.level0)
                        & !(U256::ONE << (key1 & U256::from(u8::MAX)));
                    self.level0 = value0.to_le_bytes();
                }
            }
            return true;
        }

        false
    }

    /// Finds the first `id` in the tree that is less than or equal to the given `id`.
    ///
    /// Returns the found `id`, or `U24::MAX` if there is no such `id` in the tree.
    fn find_first_right(&self, id: u32) -> u32 {
        let mut leaves: U256;

        let key2 = U256::from(id >> 8);
        let mut bit = (id & u32::from(u8::MAX)) as u8;

        if bit != 0 {
            leaves =
                U256::from_le_bytes(*self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]));
            let closest_bit = Self::_closest_bit_right(leaves, bit);

            if closest_bit != U256::MAX {
                return (key2 << 8u8).as_u32() | closest_bit.as_u32();
            }
        }

        let key1 = key2 >> 8u8;
        bit = (key2 & U256::from(u8::MAX)).as_u8();

        if bit != 0 {
            leaves =
                U256::from_le_bytes(*self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]));
            let closest_bit = Self::_closest_bit_right(leaves, bit);

            if closest_bit != U256::MAX {
                let key2 = key1 << 8u8 | closest_bit;
                leaves = U256::from_le_bytes(
                    *self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                return (key2 << 8u8).as_u32() | BitMath::most_significant_bit(leaves) as u32;
            }
        }

        bit = (key1 & U256::from(u8::MAX)).as_u8();

        if bit != 0 {
            leaves = U256::from_le_bytes(self.level0);
            let closest_bit = Self::_closest_bit_right(leaves, bit);

            if closest_bit != U256::MAX {
                let key1 = closest_bit;
                leaves = U256::from_le_bytes(
                    *self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                let key2 = key1 << 8u8 | U256::from(BitMath::most_significant_bit(leaves));
                leaves = U256::from_le_bytes(
                    *self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                return (key2 << 8u8).as_u32() | BitMath::most_significant_bit(leaves) as u32;
            }
        }

        U24::MAX
    }

    /// Finds the first `id` in the tree that is greater than or equal to the given `id`.
    ///
    /// Returns the found `id`, or `0` if there is no such `id` in the tree.
    fn find_first_left(&self, id: u32) -> u32 {
        let mut leaves: U256;

        let key2 = U256::from(id >> 8);
        let mut bit = (id & u32::from(u8::MAX)) as u8;

        if bit != u8::MAX {
            leaves =
                U256::from_le_bytes(*self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]));
            let closest_bit = Self::_closest_bit_left(leaves, bit);

            if closest_bit != U256::MAX {
                return (key2 << 8u8).as_u32() | closest_bit.as_u32();
            }
        }

        let key1 = key2 >> 8u8;
        bit = (key2 & U256::from(u8::MAX)).as_u8();

        if bit != u8::MAX {
            leaves =
                U256::from_le_bytes(*self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]));
            let closest_bit = Self::_closest_bit_left(leaves, bit);

            if closest_bit != U256::MAX {
                let key2 = key1 << 8u8 | closest_bit;
                leaves = U256::from_le_bytes(
                    *self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                return (key2 << 8u8).as_u32() | BitMath::least_significant_bit(leaves) as u32;
            }
        }

        bit = (key1 & U256::from(u8::MAX)).as_u8();

        if bit != u8::MAX {
            leaves = U256::from_le_bytes(self.level0);
            let closest_bit = Self::_closest_bit_left(leaves, bit);

            if closest_bit != U256::MAX {
                let key1 = closest_bit;
                leaves = U256::from_le_bytes(
                    *self.level1.get(&key1.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                let key2 = key1 << 8u8 | U256::from(BitMath::least_significant_bit(leaves));
                leaves = U256::from_le_bytes(
                    *self.level2.get(&key2.to_le_bytes()).unwrap_or(&[0u8; 32]),
                );

                return (key2 << 8u8).as_u32() | BitMath::least_significant_bit(leaves) as u32;
            }
        }

        0u32
    }

    /// Helper function: finds the first bit in the given `leaves` that is strictly lower than the given `bit`.
    ///
    /// Returns the found bit, or `U256::MAX` if there is no such bit.
    fn _closest_bit_right(leaves: U256, bit: u8) -> U256 {
        BitMath::closest_bit_right(leaves, bit - 1)
    }

    /// Helper function: finds the first bit in the given `leaves` that is strictly higher than the given `bit`.
    ///
    /// Returns the found bit, or `U256::MAX` if there is no such bit.
    fn _closest_bit_left(leaves: U256, bit: u8) -> U256 {
        BitMath::closest_bit_left(leaves, bit + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        let tree = MockTreeUint24::new();
        let ids: Vec<u32> = vec![1, 2, 3, 4, 5];

        for id in ids {
            let contains = tree.contains(id);
            assert!(!contains);
        }
    }

    #[test]
    fn test_add_to_tree_min() {
        let mut tree = MockTreeUint24::new();
        let ids: Vec<u32> = vec![0, 1, 2, 3, 4, 5];

        for id in ids {
            // Check if the tree already contains this ID
            assert!(!tree.contains(id));

            // Add the ID to the tree and check the return value
            assert!(tree.add(id));

            // Now the tree should contain this ID
            assert!(tree.contains(id));
        }
    }

    #[test]
    fn test_add_to_tree_max() {
        let mut tree = MockTreeUint24::new();
        let max = U24::MAX;
        let ids: Vec<u32> = vec![max - 1, max - 2, max - 3, max - 4, max - 5, max - 6];

        for id in ids {
            // Check if the tree already contains this ID
            assert!(!tree.contains(id));

            // Add the ID to the tree and check the return value
            assert!(tree.add(id));

            // Now the tree should contain this ID
            assert!(tree.contains(id));
        }
    }

    #[test]
    fn test_remove_from_tree() {
        let mut tree = MockTreeUint24::new();
        let ids: Vec<u32> = vec![0, 1, 2, 3, 4, 5];

        // First add all the ids to the tree
        for id in &ids {
            tree.add(*id);
        }

        // Now let's try removing them
        for id in ids {
            // Check if the tree contains this ID
            assert!(tree.contains(id));

            // Remove the ID from the tree and check the return value
            assert!(tree.remove(id));

            // Now the tree should not contain this ID
            assert!(!tree.contains(id));
        }
    }

    #[test]
    fn test_remove_to_tree_max() {
        let mut tree = MockTreeUint24::new();
        let max = U24::MAX;
        let ids: Vec<u32> = vec![max - 1, max - 2, max - 3, max - 4, max - 5, max - 6];

        // First add all the ids to the tree
        for id in &ids {
            tree.add(*id);
        }

        // Now let's try removing them
        for id in ids {
            // Check if the tree contains this ID
            assert!(tree.contains(id));

            // Remove the ID from the tree and check the return value
            assert!(tree.remove(id));

            // Now the tree should not contain this ID
            assert!(!tree.contains(id));
        }
    }

    #[test]
    fn test_remove_logic_and_search_right() {
        let mut tree = MockTreeUint24::new();
        let id = 3;

        tree.add(id);
        tree.add(id - 1);

        assert_eq!(
            tree.find_first_right(id),
            id - 1,
            "test_remove_logic_and_search_right::1"
        );

        tree.remove(id - 1);
        assert_eq!(
            tree.find_first_right(id),
            U24::MAX,
            "test_remove_logic_and_search_right::2"
        );
    }

    #[test]
    fn test_remove_logic_and_search_left() {
        let mut tree = MockTreeUint24::new();
        let id = U24::MAX - 1;

        tree.add(id);
        tree.add(id + 1);

        assert_eq!(
            tree.find_first_left(id),
            id + 1,
            "test_remove_logic_and_search_left::1"
        );

        tree.remove(id + 1);
        assert_eq!(
            tree.find_first_left(id),
            0,
            "test_remove_logic_and_search_left::2"
        );
    }

    #[test]
    fn test_find_first() {
        let mut tree = MockTreeUint24::new();

        tree.add(0);
        tree.add(1);
        tree.add(2);

        assert_eq!(tree.find_first_right(2), 1, "test_find_first::1");
        assert_eq!(tree.find_first_right(1), 0, "test_find_first::2");
        assert_eq!(tree.find_first_left(0), 1, "test_find_first::3");
        assert_eq!(tree.find_first_left(1), 2, "test_find_first::4");
        assert_eq!(tree.find_first_right(0), U24::MAX, "test_find_first::5");
        assert_eq!(tree.find_first_left(2), 0, "test_find_first::6");
    }

    #[test]
    fn test_find_first_far() {
        let mut tree = MockTreeUint24::new();

        tree.add(0);
        tree.add(U24::MAX); // Equivalent to type(uint24).max in Solidity

        assert_eq!(tree.find_first_right(U24::MAX), 0, "test_find_first_far::1");
        assert_eq!(tree.find_first_left(0), U24::MAX, "test_find_first_far::2");
    }

    #[test]
    fn test_fuzz_find_first() {
        let mut tree = MockTreeUint24::new();
        let ids = vec![1, 5, 10, 15, 25];

        for &id in &ids {
            tree.add(id);
        }

        for &id in &ids {
            let first_right = tree.find_first_right(id);
            let first_left = tree.find_first_left(id);

            if first_right != U24::MAX {
                assert!(tree.contains(first_right), "test_fuzz_find_first::1");
                assert!(first_right < id, "test_fuzz_find_first::2");
            }

            if first_left != 0 {
                assert!(tree.contains(first_left), "test_fuzz_find_first::3");
                assert!(first_left > id, "test_fuzz_find_first::4");
            }
        }
    }

    #[test]
    fn test_test() {
        let mut tree = MockTreeUint24::new();
        let id = 8363961;

        tree.add(id + 1);
        tree.add(id);
        tree.add(id - 1);

        tree.remove(id);

        assert_eq!(
            tree.find_first_left(id - 1),
            id + 1,
            "test_remove_logic_and_search_right::1"
        );

        assert_eq!(
            tree.find_first_right(id + 1),
            id - 1,
            "test_remove_logic_and_search_right::2"
        );
    }
}
