use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, ContractInfo, StdResult, Storage};
use lb_interfaces::{lb_factory::ILbFactory, lb_pair::ContractStatus};
use lb_libraries::{Bytes32, OracleSample, PairParameters};
// TODO: sort out viewing key strategy
use ethnum::U256;
use lb_libraries::math::{bit_math::BitMath, u24::U24};
use secret_toolkit::{
    serialization::{Bincode2, Json},
    storage::{Item, Keymap, KeymapBuilder, WithoutIter},
};
use shade_protocol::{
    swap::core::{TokenType, ViewingKey},
    Contract,
};

pub static STATE: Item<State> = Item::new(b"state");

pub static CONTRACT_STATUS: Item<ContractStatus, Json> = Item::new(b"contract_status");
pub static VIEWING_KEY: Item<ViewingKey> = Item::new(b"contract_viewing_key");

pub static FACTORY: Item<ILbFactory> = Item::new(b"lb_factory");
pub static LB_TOKEN: Item<ContractInfo> = Item::new(b"lb_token");

pub static TOKEN_X: Item<TokenType, Json> = Item::new(b"token_x");
pub static TOKEN_Y: Item<TokenType, Json> = Item::new(b"token_y");
pub static BIN_STEP: Item<u16> = Item::new(b"bin_step");

pub static BINS: Keymap<u32, Bytes32, Bincode2, WithoutIter> =
    KeymapBuilder::new(b"bin_map").without_iter().build();

pub static TREE: TreeUint24 = TreeUint24 {}; // see implementation below
pub static ORACLE: Keymap<u16, OracleSample, Bincode2, WithoutIter> =
    KeymapBuilder::new(b"oracle").without_iter().build();

pub static PARAMETERS: Item<PairParameters> = Item::new(b"pair_parameters");
pub static RESERVES: Item<Bytes32> = Item::new(b"reserves");
pub static PROTOCOL_FEES: Item<Bytes32> = Item::new(b"protocol_fees");
pub static HOOKS_PARAMETERS: Item<Bytes32> = Item::new(b"hooks_parameters");

pub static EPHEMERAL_LB_TOKEN: Item<EphemeralLbToken> = Item::new(b"ephemeral_lb_token");
pub static EPHEMERAL_FLASH_LOAN: Item<EphemeralFlashLoan> = Item::new(b"ephemeral_flash_loan");

// TODO: do we even need this?
#[cw_serde]
pub struct State {
    pub creator: Addr,
    pub admin_auth: Contract,
}

#[cw_serde]
pub struct EphemeralLbToken {
    pub code_hash: String,
}

#[cw_serde]
pub struct EphemeralFlashLoan {
    pub reserves_before: Bytes32,
    pub total_fees: Bytes32,
    pub sender: Addr,
    pub receiver: Addr,
    pub amounts: Bytes32,
}

// TODO: (maybe) We could just have one storage key that can store any epehemeral storage
// type, as long as they are serializable. But maybe it's better not to?
// example:

// pub const EPHEMERAL_STORAGE: Item<Binary> = Item::new(b"ephemeral_storage");
//
// EPHEMERAL_STORAGE.save(
//     deps.storage,
//     &to_binary(&EphemeralLbToken {
//         code_hash: msg.lb_token_implementation.code_hash,
//     })?,
// )?;

static LEVEL0: Item<Bytes32> = Item::new(b"bin_tree_level0");
static LEVEL1: Keymap<Bytes32, Bytes32, Bincode2, WithoutIter> =
    KeymapBuilder::new(b"bin_tree_level1")
        .without_iter()
        .build();
static LEVEL2: Keymap<Bytes32, Bytes32, Bincode2, WithoutIter> =
    KeymapBuilder::new(b"bin_tree_level2")
        .without_iter()
        .build();

pub struct TreeUint24 {}

impl TreeUint24 {
    pub fn contains(&self, storage: &dyn Storage, id: u32) -> bool {
        let key2 = U256::from(id) >> 8u8;

        let bucket = U256::from_le_bytes(
            LEVEL2
                .get(storage, &key2.to_le_bytes())
                .unwrap_or([0u8; 32]),
        );

        let bit_position = U256::ONE << (id & 255u32);

        (bucket & bit_position) != U256::ZERO
    }

    pub fn add(&self, storage: &mut dyn Storage, id: u32) -> StdResult<bool> {
        let key2 = U256::from(id) >> 8u8;

        let leaves = U256::from_le_bytes(
            LEVEL2
                .get(storage, &key2.to_le_bytes())
                .unwrap_or([0u8; 32]),
        );
        let new_leaves = leaves | U256::ONE << (id & u8::MAX as u32);

        if leaves != new_leaves {
            LEVEL2.insert(storage, &key2.to_le_bytes(), &new_leaves.to_le_bytes())?;

            if leaves == U256::ZERO {
                let key1 = key2 >> 8u8;
                let leaves = U256::from_le_bytes(
                    LEVEL1
                        .get(storage, &key1.to_le_bytes())
                        .unwrap_or([0u8; 32]),
                );

                let value1 = leaves | (U256::ONE << (key2 & U256::from(u8::MAX)));

                LEVEL1.insert(storage, &key1.to_le_bytes(), &value1.to_le_bytes())?;

                if leaves == U256::ZERO {
                    let value0 = U256::from_le_bytes(LEVEL0.load(storage).unwrap_or([0u8; 32]))
                        | (U256::ONE << (key1 & U256::from(u8::MAX)));
                    LEVEL0.save(storage, &value0.to_le_bytes())?;
                }
            }
            return Ok(true);
        }

        Ok(false)
    }

    pub fn remove(&self, storage: &mut dyn Storage, id: u32) -> StdResult<bool> {
        let key2 = U256::from(id) >> 8u8;

        let leaves = U256::from_le_bytes(
            LEVEL2
                .get(storage, &key2.to_le_bytes())
                .unwrap_or([0u8; 32]),
        );
        let new_leaves = leaves & !(U256::ONE << (id & u8::MAX as u32));

        if leaves != new_leaves {
            LEVEL2.insert(storage, &key2.to_le_bytes(), &new_leaves.to_le_bytes())?;

            if new_leaves == U256::ZERO {
                let key1 = key2 >> 8u8;
                let leaves = U256::from_le_bytes(
                    LEVEL1
                        .get(storage, &key1.to_le_bytes())
                        .unwrap_or([0u8; 32]),
                );

                let value1 = leaves & !(U256::ONE << (key2 & U256::from(u8::MAX)));
                LEVEL1.insert(storage, &key1.to_le_bytes(), &value1.to_le_bytes())?;

                if leaves == U256::ZERO {
                    let value0 = U256::from_le_bytes(LEVEL0.load(storage).unwrap_or([0u8; 32]))
                        & !(U256::ONE << (key1 & U256::from(u8::MAX)));
                    LEVEL0.save(storage, &value0.to_le_bytes())?;
                }
            }
            return Ok(true);
        }

        Ok(false)
    }

    /// Finds the first `id` in the tree that is less than or equal to the given `id`.
    ///
    /// Returns the found `id`, or `U24::MAX` if there is no such `id` in the tree.
    pub fn find_first_right(&self, storage: &dyn Storage, id: u32) -> u32 {
        let mut leaves: U256;

        let key2 = U256::from(id >> 8);
        let mut bit = (id & u32::from(u8::MAX)) as u8;

        if bit != 0 {
            leaves = U256::from_le_bytes(
                LEVEL2
                    .get(storage, &key2.to_le_bytes())
                    .unwrap_or([0u8; 32]),
            );
            let closest_bit = Self::_closest_bit_right(leaves, bit);

            if closest_bit != U256::MAX {
                return (key2 << 8u8).as_u32() | closest_bit.as_u32();
            }
        }

        let key1 = key2 >> 8u8;
        bit = (key2 & U256::from(u8::MAX)).as_u8();

        if bit != 0 {
            leaves = U256::from_le_bytes(
                LEVEL1
                    .get(storage, &key1.to_le_bytes())
                    .unwrap_or([0u8; 32]),
            );
            let closest_bit = Self::_closest_bit_right(leaves, bit);

            if closest_bit != U256::MAX {
                let key2 = key1 << 8u8 | closest_bit;
                leaves = U256::from_le_bytes(
                    LEVEL2
                        .get(storage, &key2.to_le_bytes())
                        .unwrap_or([0u8; 32]),
                );

                return (key2 << 8u8).as_u32() | BitMath::most_significant_bit(leaves) as u32;
            }
        }

        bit = (key1 & U256::from(u8::MAX)).as_u8();

        if bit != 0 {
            leaves = U256::from_le_bytes(LEVEL0.load(storage).unwrap_or([0u8; 32]));
            let closest_bit = Self::_closest_bit_right(leaves, bit);

            if closest_bit != U256::MAX {
                let key1 = closest_bit;
                leaves = U256::from_le_bytes(
                    LEVEL1
                        .get(storage, &key1.to_le_bytes())
                        .unwrap_or([0u8; 32]),
                );

                let key2 = key1 << 8u8 | U256::from(BitMath::most_significant_bit(leaves));
                leaves = U256::from_le_bytes(
                    LEVEL2
                        .get(storage, &key2.to_le_bytes())
                        .unwrap_or([0u8; 32]),
                );

                return (key2 << 8u8).as_u32() | BitMath::most_significant_bit(leaves) as u32;
            }
        }

        U24::MAX
    }

    /// Finds the first `id` in the tree that is greater than or equal to the given `id`.
    ///
    /// Returns the found `id`, or `0` if there is no such `id` in the tree.
    pub fn find_first_left(&self, storage: &dyn Storage, id: u32) -> u32 {
        let mut leaves: U256;

        let key2 = U256::from(id >> 8);
        let mut bit = (id & u32::from(u8::MAX)) as u8;

        if bit != u8::MAX {
            leaves = U256::from_le_bytes(
                LEVEL2
                    .get(storage, &key2.to_le_bytes())
                    .unwrap_or([0u8; 32]),
            );
            let closest_bit = Self::_closest_bit_left(leaves, bit);

            if closest_bit != U256::MAX {
                return (key2 << 8u8).as_u32() | closest_bit.as_u32();
            }
        }

        let key1 = key2 >> 8u8;
        bit = (key2 & U256::from(u8::MAX)).as_u8();

        if bit != u8::MAX {
            leaves = U256::from_le_bytes(
                LEVEL1
                    .get(storage, &key1.to_le_bytes())
                    .unwrap_or([0u8; 32]),
            );
            let closest_bit = Self::_closest_bit_left(leaves, bit);

            if closest_bit != U256::MAX {
                let key2 = key1 << 8u8 | closest_bit;
                leaves = U256::from_le_bytes(
                    LEVEL2
                        .get(storage, &key2.to_le_bytes())
                        .unwrap_or([0u8; 32]),
                );

                return (key2 << 8u8).as_u32() | BitMath::least_significant_bit(leaves) as u32;
            }
        }

        bit = (key1 & U256::from(u8::MAX)).as_u8();

        if bit != u8::MAX {
            leaves = U256::from_le_bytes(LEVEL0.load(storage).unwrap_or([0u8; 32]));
            let closest_bit = Self::_closest_bit_left(leaves, bit);

            if closest_bit != U256::MAX {
                let key1 = closest_bit;
                leaves = U256::from_le_bytes(
                    LEVEL1
                        .get(storage, &key1.to_le_bytes())
                        .unwrap_or([0u8; 32]),
                );

                let key2 = key1 << 8u8 | U256::from(BitMath::least_significant_bit(leaves));
                leaves = U256::from_le_bytes(
                    LEVEL2
                        .get(storage, &key2.to_le_bytes())
                        .unwrap_or([0u8; 32]),
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

pub trait TreeUint24Trait {
    fn contains(&self, storage: &dyn Storage, id: u32) -> bool;
    fn add(&mut self, storage: &mut dyn Storage, id: u32) -> bool;
    fn remove(&mut self, storage: &mut dyn Storage, id: u32) -> bool;
}

impl TreeUint24Trait for Item<'_, Bytes32> {
    fn contains(&self, storage: &dyn Storage, id: u32) -> bool {
        let key2 = U256::from(id) >> 8u8;

        let bucket = U256::from_le_bytes(
            LEVEL2
                .get(storage, &key2.to_le_bytes())
                .unwrap_or([0u8; 32]),
        );

        let bit_position = U256::ONE << (id & 255u32);

        (bucket & bit_position) != U256::ZERO
    }

    fn add(&mut self, storage: &mut dyn Storage, id: u32) -> bool {
        let key2 = U256::from(id) >> 8u8;

        let leaves = U256::from_le_bytes(
            LEVEL2
                .get(storage, &key2.to_le_bytes())
                .unwrap_or([0u8; 32]),
        );
        let new_leaves = leaves | U256::ONE << (id & u8::MAX as u32);

        if leaves != new_leaves {
            LEVEL2
                .insert(storage, &key2.to_le_bytes(), &new_leaves.to_le_bytes())
                .expect("why would this ever fail?");

            if leaves == U256::ZERO {
                let key1 = key2 >> 8u8;
                let leaves = U256::from_le_bytes(
                    LEVEL1
                        .get(storage, &key1.to_le_bytes())
                        .unwrap_or([0u8; 32]),
                );

                let value1 = leaves | (U256::ONE << (key2 & U256::from(u8::MAX)));

                LEVEL1
                    .insert(storage, &key1.to_le_bytes(), &value1.to_le_bytes())
                    .expect("why would this ever fail?");

                if leaves == U256::ZERO {
                    let value0 = U256::from_le_bytes(self.load(storage).unwrap())
                        | (U256::ONE << (key1 & U256::from(u8::MAX)));
                    self.save(storage, &value0.to_le_bytes()).unwrap();
                }
            }
            return true;
        }

        false
    }

    fn remove(&mut self, storage: &mut dyn Storage, id: u32) -> bool {
        let key2 = U256::from(id) >> 8u8;

        let leaves = U256::from_le_bytes(
            LEVEL2
                .get(storage, &key2.to_le_bytes())
                .unwrap_or([0u8; 32]),
        );
        let new_leaves = leaves & !(U256::ONE << (id & u8::MAX as u32));

        if leaves != new_leaves {
            LEVEL2
                .insert(storage, &key2.to_le_bytes(), &new_leaves.to_le_bytes())
                .expect("why would this ever fail?");

            if new_leaves == U256::ZERO {
                let key1 = key2 >> 8u8;
                let leaves = U256::from_le_bytes(
                    LEVEL1
                        .get(storage, &key1.to_le_bytes())
                        .unwrap_or([0u8; 32]),
                );

                let value1 = leaves & !(U256::ONE << (key2 & U256::from(u8::MAX)));
                LEVEL1
                    .insert(storage, &key1.to_le_bytes(), &value1.to_le_bytes())
                    .expect("why would this ever fail?");

                if leaves == U256::ZERO {
                    let value0 = U256::from_le_bytes(self.load(storage).unwrap())
                        & !(U256::ONE << (key1 & U256::from(u8::MAX)));
                    self.save(storage, &value0.to_le_bytes()).unwrap();
                }
            }
            return true;
        }

        false
    }
}
