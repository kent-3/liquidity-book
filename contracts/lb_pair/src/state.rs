use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, ContractInfo, StdError, StdResult, Storage};
use lb_interfaces::lb_factory::ILbFactory;
use lb_interfaces::lb_pair::ContractStatus;
use lb_libraries::{Bytes32, OracleSample, PairParameters, TreeUint24};
// TODO: sort out viewing key strategy
use shade_protocol::swap::core::ViewingKey;
use shade_protocol::{
    secret_storage_plus::{Bincode2, Item, Map},
    swap::core::TokenType,
    Contract,
};

// TODO: exploring this idea

pub trait BinTree {
    fn add(&self, storage: &mut dyn Storage, id: u32) -> Result<bool, StdError>;
}

impl BinTree for Item<'_, TreeUint24, Bincode2> {
    fn add(&self, storage: &mut dyn Storage, id: u32) -> Result<bool, StdError> {
        let mut result: bool = true;

        BIN_TREE.update(storage, |mut tree| -> StdResult<_> {
            result = tree.add(id);
            Ok(tree)
        })?;

        Ok(result)
    }
}
pub const STATE: Item<State> = Item::new("state");
pub const CONTRACT_STATUS: Item<ContractStatus> = Item::new("contract_status");
pub const VIEWING_KEY: Item<ViewingKey> = Item::new("contract_viewing_key");

pub const FACTORY: Item<ILbFactory> = Item::new("factory");
pub const LB_TOKEN: Item<ContractInfo> = Item::new("lb_token");

pub const TOKEN_X: Item<TokenType> = Item::new("token_x");
pub const TOKEN_Y: Item<TokenType> = Item::new("token_y");
pub const BIN_STEP: Item<u16> = Item::new("bin_step");

pub const BIN_MAP: Map<u32, Bytes32> = Map::new("bin_map");
pub const BIN_TREE: Item<TreeUint24, Bincode2> = Item::new("bin_tree");
pub const ORACLE: Map<u16, OracleSample> = Map::new("oracle");

pub const PARAMETERS: Item<PairParameters> = Item::new("pair_parameters");
pub const RESERVES: Item<Bytes32> = Item::new("reserves");
pub const PROTOCOL_FEES: Item<Bytes32> = Item::new("protocol_fees");
pub const HOOKS_PARAMETERS: Item<Bytes32> = Item::new("hooks_parameters");

pub const EPHEMERAL_LB_TOKEN: Item<EphemeralLbToken> = Item::new("ephemeral_lb_token");
pub const EPHEMERAL_FLASH_LOAN: Item<EphemeralFlashLoan> = Item::new("ephemeral_flash_loan");

// TODO: clean this up
#[cw_serde]
pub struct State {
    pub creator: Addr,
    // pub viewing_key: ViewingKey,
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

// TODO: (maybe) We could just have one storage key that can store any of the epehemeral storage
// types, as long as they are serializable. But maybe it's better not to?
// example:
//     EPHEMERAL_STORAGE.save(
//     deps.storage,
//     &to_binary(&EphemeralLbToken {
//         code_hash: msg.lb_token_implementation.code_hash,
//     })?,
// )?;

pub const EPHEMERAL_STORAGE: Item<Binary> = Item::new("ephemeral_storage");

// experimental!

use ethnum::U256;
use lb_libraries::math::bit_math::BitMath;
use lb_libraries::math::u24::U24;
use secret_toolkit::storage::{Item as Item2, Keymap, KeymapBuilder, WithoutIter};

pub static TREE: TreeUint24_ = TreeUint24_ {};
pub static LEVEL0: Item2<Bytes32> = Item2::new(b"bin_tree_level0");
pub static LEVEL1: Keymap<Bytes32, Bytes32, secret_toolkit::serialization::Bincode2, WithoutIter> =
    KeymapBuilder::new(b"bin_tree_level1")
        .without_iter()
        .build();
pub static LEVEL2: Keymap<Bytes32, Bytes32, secret_toolkit::serialization::Bincode2, WithoutIter> =
    KeymapBuilder::new(b"bin_tree_level2")
        .without_iter()
        .build();

pub struct TreeUint24_ {}

impl TreeUint24_ {
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
