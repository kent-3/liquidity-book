//! ### Liquidity Book Oracle Helper Library
//! Author: Kent
//!
//! This library contains functions to manage the oracle.
//! The oracle samples are stored in a HashMap with 65535 possible entries.
//!
//! Each sample is encoded as follows:
//! * 0 - 16: oracle length (16 bits)
//! * 16 - 80: cumulative id (64 bits)
//! * 80 - 144: cumulative volatility accumulator (64 bits)
//! * 144 - 208: cumulative bin crossed (64 bits)
//! * 208 - 216: sample lifetime (8 bits)
//! * 216 - 256: sample creation timestamp (40 bits)

use crate::{
    math::{sample_math::OracleSample, u256x256_math::addmod},
    pair_parameter_helper::PairParameters,
};
use cosmwasm_std::Storage;
use ethnum::U256;
use secret_toolkit::{serialization::Bincode2, storage::WithoutIter};
use std::cmp::Ordering::{Equal, Greater, Less};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum OracleError {
    #[error("Oracle Error: Invalid Oracle ID")]
    InvalidOracleId,
    #[error("Oracle Error: New length too small")]
    NewLengthTooSmall,
    #[error("Oracle Error: Lookup timestamp too old")]
    LookUpTimestampTooOld,
}

impl From<OracleError> for StdError {
    fn from(value: OracleError) -> Self {
        StdError::GenericErr {
            msg: value.to_string(),
        }
    }
}

/// This represents a fixed-size storage for 65535 samples,
/// where each sample is a 32-byte (256-bit) value.
// #[derive(Serialize, Deserialize)]
// pub struct Oracle {
//     pub samples: HashMap<u16, OracleSample>,
// }

pub const MAX_SAMPLE_LIFETIME: u8 = 120; //seconds

// impl Oracle {
//     /// Modifier to check that the oracle id is valid.
//     fn check_oracle_id(oracle_id: u16) -> Result<(), OracleError> {
//         if oracle_id == 0 {
//             return Err(OracleError::InvalidOracleId);
//         }
//
//         Ok(())
//     }
//
//     /// Returns the sample at the given oracleId.
//     pub fn get_sample(&self, oracle_id: u16) -> Result<OracleSample, OracleError> {
//         Self::check_oracle_id(oracle_id)?;
//
//         // TODO - Should this return a default sample if there is None? or an Error?
//         match self.samples.get(&(oracle_id - 1)) {
//             Some(sample) => Ok(*sample),
//             None => Ok(OracleSample::default()),
//         }
//     }
//
//     /// Returns the active sample (Bytes32) and the active size (u16) of the oracle.
//     pub fn get_active_sample_and_size(
//         &self,
//         oracle_id: u16,
//     ) -> Result<(OracleSample, u16), OracleError> {
//         let active_sample = self.get_sample(oracle_id)?;
//         let mut active_size = OracleSample::get_oracle_length(&active_sample);
//
//         if oracle_id != active_size {
//             active_size = OracleSample::get_oracle_length(&self.get_sample(active_size)?);
//             active_size = if oracle_id > active_size {
//                 oracle_id
//             } else {
//                 active_size
//             };
//         }
//
//         Ok((active_sample, active_size))
//     }
//
//     /// Returns the sample at the given timestamp. If the timestamp is not in the oracle, it returns the closest sample.
//     ///
//     /// # Arguments
//     ///
//     /// * `oracle_id` - The oracle id
//     /// * `look_up_timestamp` - The timestamp to look up
//     ///
//     /// # Returns
//     ///
//     /// * `last_update` - The last update timestamp
//     /// * `cumulative_id` - The cumulative id
//     /// * `cumulative_volatility` - The cumulative volatility
//     /// * `cumulative_bin_crossed` - The cumulative bin crossed
//     pub fn get_sample_at(
//         &self,
//         oracle_id: u16,
//         look_up_timestamp: u64,
//     ) -> Result<(u64, u64, u64, u64), OracleError> {
//         let (active_sample, active_size) = self.get_active_sample_and_size(oracle_id)?;
//
//         if OracleSample::get_sample_last_update(&self.samples[&(oracle_id % active_size)])
//             > look_up_timestamp
//         {
//             return Err(OracleError::LookUpTimestampTooOld);
//         }
//
//         let mut last_update = OracleSample::get_sample_last_update(&active_sample);
//         if last_update <= look_up_timestamp {
//             return Ok((
//                 last_update,
//                 OracleSample::get_cumulative_id(&active_sample),
//                 OracleSample::get_cumulative_volatility(&active_sample),
//                 OracleSample::get_cumulative_bin_crossed(&active_sample),
//             ));
//         } else {
//             last_update = look_up_timestamp;
//         }
//         let (prev_sample, next_sample) =
//             self.binary_search(oracle_id, look_up_timestamp, active_size)?;
//         let weight_prev = next_sample.get_sample_last_update() - look_up_timestamp;
//         let weight_next = look_up_timestamp - prev_sample.get_sample_last_update();
//
//         let (cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
//             OracleSample::get_weighted_average(prev_sample, next_sample, weight_prev, weight_next);
//
//         Ok((
//             last_update,
//             cumulative_id,
//             cumulative_volatility,
//             cumulative_bin_crossed,
//         ))
//     }
//
//     /// Binary search to find the 2 samples surrounding the given timestamp.
//     ///
//     /// # Arguments
//     ///
//     /// * `oracle` - The oracle
//     /// * `oracleId` - The oracle id
//     /// * `look_up_timestamp` - The timestamp to look up
//     /// * `length` - The oracle length
//     ///
//     /// # Returns
//     ///
//     /// * `prev_sample` - The previous sample
//     /// * `next_sample` - The next sample
//     pub fn binary_search(
//         &self,
//         oracle_id: u16,
//         look_up_timestamp: u64,
//         length: u16,
//     ) -> Result<(OracleSample, OracleSample), OracleError> {
//         let mut oracle_id = oracle_id;
//         let mut low = 0;
//         let mut high = length - 1;
//
//         // TODO: not sure if it's ok to initialize these at 0
//         let mut sample = OracleSample::default();
//         let mut sample_last_update = 0u64;
//
//         let start_id = oracle_id; // oracleId is 1-based
//
//         while low <= high {
//             let mid = ((low as u32 + high as u32) >> 1) as u16;
//
//             oracle_id = ((start_id as u32 + mid as u32) % (length as u32)) as u16;
//
//             sample = *self
//                 .samples
//                 .get(&oracle_id)
//                 .unwrap_or(&OracleSample::default());
//
//             sample_last_update = sample.get_sample_last_update();
//
//             match sample_last_update.cmp(&look_up_timestamp) {
//                 Ordering::Greater => high = mid - 1,
//                 Ordering::Less => low = mid + 1,
//                 Ordering::Equal => return Ok((sample, sample)),
//             }
//         }
//
//         if look_up_timestamp < sample_last_update {
//             if oracle_id == 0 {
//                 oracle_id = length;
//             }
//
//             let prev_sample = *self
//                 .samples
//                 .get(&(oracle_id - 1))
//                 .unwrap_or(&OracleSample::default());
//
//             Ok((prev_sample, sample))
//         } else {
//             oracle_id = addmod(oracle_id.into(), U256::ONE, length.into()).as_u16();
//
//             let next_sample = *self
//                 .samples
//                 .get(&oracle_id)
//                 .unwrap_or(&OracleSample::default());
//
//             Ok((sample, next_sample))
//         }
//     }
//
//     /// Sets the sample at the given oracle_id.
//     pub fn set_sample(&mut self, oracle_id: u16, sample: OracleSample) -> Result<(), OracleError> {
//         Self::check_oracle_id(oracle_id)?;
//
//         self.samples.insert(oracle_id - 1, sample);
//
//         Ok(())
//     }
//
//     /// Updates the oracle and returns the updated pair parameters.
//     pub fn update(
//         &mut self,
//         time: &Timestamp,
//         mut parameters: PairParameters,
//         active_id: u32, // this is the new active_id
//     ) -> Result<PairParameters, OracleError> {
//         let mut oracle_id = parameters.get_oracle_id();
//         if oracle_id == 0 {
//             return Ok(parameters);
//         }
//
//         let sample = self.get_sample(oracle_id)?;
//
//         let mut created_at = sample.get_sample_creation();
//         let last_updated_at = created_at + sample.get_sample_lifetime() as u64;
//
//         if time.seconds() > last_updated_at {
//             let (cumulative_id, cumulative_volatility, cumulative_bin_crossed) = sample.update(
//                 time.seconds() - last_updated_at,
//                 parameters.get_active_id(), // this gets the current active_id
//                 parameters.get_volatility_accumulator(),
//                 parameters.get_delta_id(active_id),
//             );
//
//             let length = sample.get_oracle_length();
//             let lifetime = time.seconds() - created_at;
//
//             if lifetime > MAX_SAMPLE_LIFETIME as u64 {
//                 oracle_id = (oracle_id % length) + 1;
//                 created_at = time.seconds();
//             }
//
//             let new_sample = OracleSample::encode(
//                 length,
//                 cumulative_id,
//                 cumulative_volatility,
//                 cumulative_bin_crossed,
//                 lifetime as u8,
//                 created_at,
//             );
//
//             self.set_sample(oracle_id, new_sample)?;
//
//             parameters.set_oracle_id(oracle_id);
//
//             return Ok(parameters);
//         }
//
//         Ok(parameters)
//     }
//
//     /// Increases the oracle length.
//     pub fn increase_length(
//         &mut self,
//         oracle_id: u16,
//         new_length: u16,
//     ) -> Result<&mut Self, OracleError> {
//         let sample = self.get_sample(oracle_id)?;
//         let length = sample.get_oracle_length();
//
//         if length >= new_length {
//             return Err(OracleError::NewLengthTooSmall);
//         }
//
//         let last_sample = if length == oracle_id {
//             sample
//         } else if length == 0 {
//             OracleSample::default()
//         } else {
//             self.get_sample(length)?
//         };
//
//         let mut active_size = last_sample.get_oracle_length();
//         active_size = if oracle_id > active_size {
//             oracle_id
//         } else {
//             active_size
//         };
//
//         for i in length..new_length {
//             // NOTE: I think what this does is encode the active_size as the oracle_length (16 bits)
//             // in each of the newly added samples... the rest of the sample values are empty.
//             self.samples
//                 .insert(i, OracleSample(U256::from(active_size).to_le_bytes()));
//         }
//
//         // I think this is a fancy way of changing the length of the current sample.
//         // It's confusing looking because we don't have methods for pow or bitOR for bytes32,
//         // so we have to convert to U256 and back.
//         let new_sample =
//             (U256::from_le_bytes(sample.0) ^ U256::from(length)) | U256::from(new_length);
//
//         self.set_sample(oracle_id, OracleSample(new_sample.to_le_bytes()))?;
//
//         Ok(self)
//     }
// }

use cosmwasm_std::StdError;

pub trait OracleMap {
    fn check_oracle_id(oracle_id: u16) -> Result<(), StdError>;
    fn get_sample(&self, storage: &dyn Storage, oracle_id: u16) -> Result<OracleSample, StdError>;
    fn get_active_sample_and_size(
        &self,
        storage: &dyn Storage,
        oracle_id: u16,
    ) -> Result<(OracleSample, u16), StdError>;
    fn get_sample_at(
        &self,
        storage: &dyn Storage,
        oracle_id: u16,
        look_up_timestamp: u64,
    ) -> Result<(u64, u64, u64, u64), StdError>;
    fn binary_search(
        &self,
        storage: &dyn Storage,
        oracle_id: u16,
        look_up_timestamp: u64,
        length: u16,
    ) -> Result<(OracleSample, OracleSample), StdError>;
    fn set_sample(
        &self,
        storage: &mut dyn Storage,
        oracle_id: u16,
        sample: OracleSample,
    ) -> Result<(), StdError>;
    fn update_oracle(
        &self,
        storage: &mut dyn Storage,
        time: u64,
        parameters: PairParameters,
        active_id: u32, // this is the new active_id
    ) -> Result<PairParameters, StdError>;
    fn increase_length(
        &self,
        storage: &mut dyn Storage,
        oracle_id: u16,
        new_length: u16,
    ) -> Result<(), StdError>;
}

impl OracleMap for secret_toolkit::storage::Keymap<'_, u16, OracleSample, Bincode2, WithoutIter> {
    /// Modifier to check that the oracle id is valid.
    fn check_oracle_id(oracle_id: u16) -> Result<(), StdError> {
        if oracle_id == 0 {
            return Err(OracleError::InvalidOracleId.into());
        }

        Ok(())
    }

    /// Returns the sample at the given oracleId.
    fn get_sample(&self, storage: &dyn Storage, oracle_id: u16) -> Result<OracleSample, StdError> {
        Self::check_oracle_id(oracle_id)?;

        Ok(self.get(storage, &(oracle_id - 1)).unwrap_or_default())
    }

    fn get_active_sample_and_size(
        &self,
        storage: &dyn Storage,
        oracle_id: u16,
    ) -> Result<(OracleSample, u16), StdError> {
        let active_sample = self.get_sample(storage, oracle_id)?;
        let mut active_size = active_sample.get_oracle_length();

        if oracle_id != active_size {
            active_size = self.get_sample(storage, active_size)?.get_oracle_length();
            active_size = if oracle_id > active_size {
                oracle_id
            } else {
                active_size
            };
        }

        Ok((active_sample, active_size))
    }

    fn get_sample_at(
        &self,
        storage: &dyn Storage,
        oracle_id: u16,
        look_up_timestamp: u64,
    ) -> Result<(u64, u64, u64, u64), StdError> {
        let (active_sample, active_size) = self.get_active_sample_and_size(storage, oracle_id)?;

        // if OracleSample::get_sample_last_update(&self.samples[&(oracle_id % active_size)])
        if self
            .get(storage, &(oracle_id % active_size))
            .unwrap_or_default() // this will return all zeroes
            .get_sample_last_update() // so this will also return zero
            > look_up_timestamp
        {
            return Err(StdError::generic_err(
                OracleError::LookUpTimestampTooOld.to_string(),
            ));
        }

        let mut last_update = active_sample.get_sample_last_update();
        if last_update <= look_up_timestamp {
            return Ok((
                last_update,
                active_sample.get_cumulative_id(),
                active_sample.get_cumulative_volatility(),
                active_sample.get_cumulative_bin_crossed(),
            ));
        } else {
            last_update = look_up_timestamp;
        }
        let (prev_sample, next_sample) =
            self.binary_search(storage, oracle_id, look_up_timestamp, active_size)?;
        println!("look_up_timestamp: {:?}", look_up_timestamp);
        println!(
            "next_sample last update: {:?}",
            next_sample.get_sample_last_update()
        );
        println!(
            "prev_sample last update: {:?}",
            prev_sample.get_sample_last_update()
        );
        println!(
            "prev_sample creation: {:?}",
            prev_sample.get_sample_creation()
        );
        let weight_prev = next_sample.get_sample_last_update() - look_up_timestamp;
        let weight_next = look_up_timestamp - prev_sample.get_sample_last_update();
        println!("weight_prev: {:?}", weight_prev);
        println!("weight_next: {:?}", weight_next);

        let (cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
            OracleSample::get_weighted_average(prev_sample, next_sample, weight_prev, weight_next);
        println!("cumulative_id: {:?}", cumulative_id);

        Ok((
            last_update,
            cumulative_id,
            cumulative_volatility,
            cumulative_bin_crossed,
        ))
    }

    fn binary_search(
        &self,
        storage: &dyn Storage,
        mut oracle_id: u16,
        look_up_timestamp: u64,
        length: u16,
    ) -> Result<(OracleSample, OracleSample), StdError> {
        let mut low = U256::ZERO;
        let mut high = U256::from(length - 1);

        let mut sample = OracleSample::default();
        let mut sample_last_update = 0;

        let start_id = U256::from(oracle_id); // oracleId is 1-based
        while low <= high {
            let mid: U256 = (low + high) >> 1;

            // TODO: check if this is the best way to do this
            oracle_id = addmod(start_id, mid, length.into()).as_u16();

            sample = self.get(storage, &oracle_id).unwrap_or_default();
            sample_last_update = sample.get_sample_last_update();

            match sample_last_update.cmp(&look_up_timestamp) {
                Greater => high = mid - 1,
                Less => low = mid + 1,
                Equal => return Ok((sample, sample)),
            }
        }

        if look_up_timestamp < sample_last_update {
            if oracle_id == 0 {
                oracle_id = length;
            }

            let prev_sample = self.get(storage, &(oracle_id - 1)).unwrap_or_default();

            Ok((prev_sample, sample))
        } else {
            oracle_id = addmod(oracle_id.into(), U256::ONE, length.into()).as_u16();

            let next_sample = self.get(storage, &oracle_id).unwrap_or_default();

            Ok((sample, next_sample))
        }
    }

    fn set_sample(
        &self,
        storage: &mut dyn Storage,
        oracle_id: u16,
        sample: OracleSample,
    ) -> Result<(), StdError> {
        Self::check_oracle_id(oracle_id)?;

        self.insert(storage, &(oracle_id - 1), &sample)
    }

    fn update_oracle(
        &self,
        storage: &mut dyn Storage,
        time: u64,
        mut parameters: PairParameters,
        active_id: u32, // this is the new active_id
    ) -> Result<PairParameters, StdError> {
        let mut oracle_id = parameters.get_oracle_id();
        if oracle_id == 0 {
            return Ok(parameters);
        }

        let sample = self.get_sample(storage, oracle_id)?;

        let mut created_at = sample.get_sample_creation();
        let last_updated_at = created_at + sample.get_sample_lifetime() as u64;

        if time > last_updated_at {
            let (cumulative_id, cumulative_volatility, cumulative_bin_crossed) = sample.update(
                time - last_updated_at,
                parameters.get_active_id(), // this gets the current active_id
                parameters.get_volatility_accumulator(),
                parameters.get_delta_id(active_id),
            );

            let length = sample.get_oracle_length();
            let lifetime = time - created_at;

            if lifetime > MAX_SAMPLE_LIFETIME as u64 {
                oracle_id = (oracle_id % length) + 1;
                created_at = time;
            }

            let new_sample = OracleSample::encode(
                length,
                cumulative_id,
                cumulative_volatility,
                cumulative_bin_crossed,
                lifetime as u8,
                created_at,
            );

            self.set_sample(storage, oracle_id, new_sample)?;

            parameters.set_oracle_id(oracle_id);

            return Ok(parameters);
        }

        Ok(parameters)
    }

    fn increase_length(
        &self,
        storage: &mut dyn Storage,
        oracle_id: u16,
        new_length: u16,
    ) -> Result<(), StdError> {
        let sample = self.get_sample(storage, oracle_id)?;
        let length = sample.get_oracle_length();

        if length >= new_length {
            return Err(StdError::generic_err(
                OracleError::NewLengthTooSmall.to_string(),
            ));
        }

        let last_sample = if length == oracle_id {
            sample
        } else if length == 0 {
            OracleSample::default()
        } else {
            self.get_sample(storage, length)?
        };

        let mut active_size = last_sample.get_oracle_length();
        active_size = if oracle_id > active_size {
            oracle_id
        } else {
            active_size
        };

        // New samples are initialized and stored. The only information they contain is the length of the oracle.
        for i in length..new_length {
            self.insert(storage, &i, &OracleSample::from(active_size))?;
        }

        let new_sample =
            OracleSample::from((U256::from(sample) ^ U256::from(length)) | U256::from(new_length));

        self.set_sample(storage, oracle_id, new_sample)?;

        Ok(())
    }
}

impl OracleMap for secret_storage_plus::Map<'_, u16, OracleSample> {
    /// Modifier to check that the oracle id is valid.
    fn check_oracle_id(oracle_id: u16) -> Result<(), StdError> {
        if oracle_id == 0 {
            return Err(OracleError::InvalidOracleId.into());
        }

        Ok(())
    }

    /// Returns the sample at the given oracleId.
    fn get_sample(&self, storage: &dyn Storage, oracle_id: u16) -> Result<OracleSample, StdError> {
        Self::check_oracle_id(oracle_id)?;

        Ok(self.load(storage, oracle_id - 1).unwrap_or_default())
    }

    fn get_active_sample_and_size(
        &self,
        storage: &dyn Storage,
        oracle_id: u16,
    ) -> Result<(OracleSample, u16), StdError> {
        let active_sample = self.get_sample(storage, oracle_id)?;
        let mut active_size = active_sample.get_oracle_length();

        if oracle_id != active_size {
            active_size = self.get_sample(storage, active_size)?.get_oracle_length();
            active_size = if oracle_id > active_size {
                oracle_id
            } else {
                active_size
            };
        }

        Ok((active_sample, active_size))
    }

    fn get_sample_at(
        &self,
        storage: &dyn Storage,
        oracle_id: u16,
        look_up_timestamp: u64,
    ) -> Result<(u64, u64, u64, u64), StdError> {
        let (active_sample, active_size) = self.get_active_sample_and_size(storage, oracle_id)?;

        // if OracleSample::get_sample_last_update(&self.samples[&(oracle_id % active_size)])
        if self
            .load(storage, oracle_id % active_size)
            .unwrap_or_default()
            .get_sample_last_update()
            > look_up_timestamp
        {
            return Err(StdError::generic_err(
                OracleError::LookUpTimestampTooOld.to_string(),
            ));
        }

        let mut last_update = active_sample.get_sample_last_update();
        if last_update <= look_up_timestamp {
            return Ok((
                last_update,
                active_sample.get_cumulative_id(),
                active_sample.get_cumulative_volatility(),
                active_sample.get_cumulative_bin_crossed(),
            ));
        } else {
            last_update = look_up_timestamp;
        }
        let (prev_sample, next_sample) =
            self.binary_search(storage, oracle_id, look_up_timestamp, active_size)?;
        println!("look_up_timestamp: {:?}", look_up_timestamp);
        println!(
            "next_sample last update: {:?}",
            next_sample.get_sample_last_update()
        );
        println!(
            "prev_sample last update: {:?}",
            prev_sample.get_sample_last_update()
        );
        println!(
            "prev_sample creation: {:?}",
            prev_sample.get_sample_creation()
        );
        let weight_prev = next_sample.get_sample_last_update() - look_up_timestamp;
        let weight_next = look_up_timestamp - prev_sample.get_sample_last_update();
        println!("weight_prev: {:?}", weight_prev);
        println!("weight_next: {:?}", weight_next);

        let (cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
            OracleSample::get_weighted_average(prev_sample, next_sample, weight_prev, weight_next);
        println!("cumulative_id: {:?}", cumulative_id);

        Ok((
            last_update,
            cumulative_id,
            cumulative_volatility,
            cumulative_bin_crossed,
        ))
    }

    fn binary_search(
        &self,
        storage: &dyn Storage,
        mut oracle_id: u16,
        look_up_timestamp: u64,
        length: u16,
    ) -> Result<(OracleSample, OracleSample), StdError> {
        let mut low = U256::ZERO;
        let mut high = U256::from(length - 1);

        let mut sample = OracleSample::default();
        let mut sample_last_update = 0;

        let start_id = U256::from(oracle_id); // oracleId is 1-based
        while low <= high {
            let mid: U256 = (low + high) >> 1;

            // TODO: check if this is the best way to do this
            oracle_id = addmod(start_id, mid, length.into()).as_u16();

            sample = self.load(storage, oracle_id).unwrap_or_default();
            sample_last_update = sample.get_sample_last_update();

            match sample_last_update.cmp(&look_up_timestamp) {
                Greater => high = mid - 1,
                Less => low = mid + 1,
                Equal => return Ok((sample, sample)),
            }
        }

        if look_up_timestamp < sample_last_update {
            if oracle_id == 0 {
                oracle_id = length;
            }

            let prev_sample = self.load(storage, oracle_id - 1).unwrap_or_default();

            Ok((prev_sample, sample))
        } else {
            oracle_id = addmod(oracle_id.into(), U256::ONE, length.into()).as_u16();

            let next_sample = self.load(storage, oracle_id).unwrap_or_default();

            Ok((sample, next_sample))
        }
    }

    fn set_sample(
        &self,
        storage: &mut dyn Storage,
        oracle_id: u16,
        sample: OracleSample,
    ) -> Result<(), StdError> {
        Self::check_oracle_id(oracle_id)?;

        self.save(storage, oracle_id - 1, &sample)
    }

    fn update_oracle(
        &self,
        storage: &mut dyn Storage,
        time: u64,
        mut parameters: PairParameters,
        active_id: u32, // this is the new active_id
    ) -> Result<PairParameters, StdError> {
        let mut oracle_id = parameters.get_oracle_id();
        if oracle_id == 0 {
            return Ok(parameters);
        }

        let sample = self.get_sample(storage, oracle_id)?;

        let mut created_at = sample.get_sample_creation();
        let last_updated_at = created_at + sample.get_sample_lifetime() as u64;

        if time > last_updated_at {
            let (cumulative_id, cumulative_volatility, cumulative_bin_crossed) = sample.update(
                time - last_updated_at,
                parameters.get_active_id(), // this gets the current active_id
                parameters.get_volatility_accumulator(),
                parameters.get_delta_id(active_id),
            );

            let length = sample.get_oracle_length();
            let lifetime = time - created_at;

            if lifetime > MAX_SAMPLE_LIFETIME as u64 {
                oracle_id = (oracle_id % length) + 1;
                created_at = time;
            }

            let new_sample = OracleSample::encode(
                length,
                cumulative_id,
                cumulative_volatility,
                cumulative_bin_crossed,
                lifetime as u8,
                created_at,
            );

            self.set_sample(storage, oracle_id, new_sample)?;

            parameters.set_oracle_id(oracle_id);

            return Ok(parameters);
        }

        Ok(parameters)
    }

    fn increase_length(
        &self,
        storage: &mut dyn Storage,
        oracle_id: u16,
        new_length: u16,
    ) -> Result<(), StdError> {
        let sample = self.get_sample(storage, oracle_id)?;
        let length = sample.get_oracle_length();

        if length >= new_length {
            return Err(StdError::generic_err(
                OracleError::NewLengthTooSmall.to_string(),
            ));
        }

        let last_sample = if length == oracle_id {
            sample
        } else if length == 0 {
            OracleSample::default()
        } else {
            self.get_sample(storage, length)?
        };

        let mut active_size = last_sample.get_oracle_length();
        active_size = if oracle_id > active_size {
            oracle_id
        } else {
            active_size
        };

        // New samples are initialized and stored. The only information they contain is the length of the oracle.
        for i in length..new_length {
            self.save(storage, i, &OracleSample::from(active_size))?;
        }

        let new_sample =
            OracleSample::from((U256::from(sample) ^ U256::from(length)) | U256::from(new_length));

        self.set_sample(storage, oracle_id, new_sample)?;

        Ok(())
    }
}

// TODO: All tests need to be modified to use a secret_storage_plus::Map somehow.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::encoded::MASK_UINT20;
    use cosmwasm_std::testing::mock_dependencies;
    use secret_storage_plus::Map;

    // Helper function to bound a value within a range
    fn bound<T: Ord>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    #[test]
    fn test_set_and_get_sample() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        // TODO: Replace with random values for fuzz testing
        let oracle_id: u16 = 1;
        let sample = OracleSample([0u8; 32]);

        oracle.set_sample(storage, oracle_id, sample).unwrap();

        let retrieved_sample = oracle.get_sample(storage, oracle_id).unwrap();
        assert_eq!(retrieved_sample, sample, "test_SetSample::1");
    }

    #[test]
    fn test_revert_set_and_get_sample() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        let oracle_id: u16 = 0;
        let sample = OracleSample([0u8; 32]);

        assert_eq!(
            oracle.set_sample(storage, oracle_id, sample),
            Err(StdError::GenericErr {
                msg: OracleError::InvalidOracleId.to_string()
            }),
            "test_revert::SetSample"
        );

        assert_eq!(
            oracle.get_sample(storage, oracle_id),
            Err(StdError::GenericErr {
                msg: OracleError::InvalidOracleId.to_string()
            }),
            "test_revert::GetSample"
        );
    }

    #[test]
    fn test_set_and_get_sample_edge_cases() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        // Test with maximum oracle_id value for u16
        let max_oracle_id: u16 = u16::MAX;
        let sample = OracleSample([1u8; 32]);

        // Set sample with maximum oracle_id
        oracle.set_sample(storage, max_oracle_id, sample).unwrap();

        // Retrieve and validate
        let retrieved_sample = oracle.get_sample(storage, max_oracle_id).unwrap();
        assert_eq!(
            retrieved_sample, sample,
            "test_set_and_get_sample_edge_cases::MaxOracleId"
        );

        // Test with minimum valid oracle_id (1, since 0 is considered invalid)
        let min_valid_oracle_id: u16 = 1;
        oracle
            .set_sample(storage, min_valid_oracle_id, sample)
            .unwrap();

        // Retrieve and validate
        let retrieved_sample = oracle.get_sample(storage, min_valid_oracle_id).unwrap();
        assert_eq!(
            retrieved_sample, sample,
            "test_set_and_get_sample_edge_cases::MinValidOracleId"
        );

        // Test with an empty sample ([0u8; 32])
        let empty_sample = OracleSample([0u8; 32]);
        oracle
            .set_sample(storage, min_valid_oracle_id, empty_sample)
            .unwrap();

        // Retrieve and validate
        let retrieved_sample = oracle.get_sample(storage, min_valid_oracle_id).unwrap();
        assert_eq!(
            retrieved_sample, empty_sample,
            "test_set_and_get_sample_edge_cases::EmptySample"
        );
    }

    #[test]
    fn test_binary_search_simple() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        let sample1 = OracleSample::encode(3, 1, 2, 3, 0, 0);
        let sample2 = OracleSample::encode(3, 2, 3, 4, 0, 10);
        let sample3 = OracleSample::encode(3, 3, 4, 5, 0, 20);

        oracle.set_sample(storage, 1, sample1).unwrap();
        oracle.set_sample(storage, 2, sample2).unwrap();
        oracle.set_sample(storage, 3, sample3).unwrap();

        let (previous, next) = oracle.binary_search(storage, 3, 0, 3).unwrap();
        assert_eq!(previous, sample1, "test_binarySearch::1");
        assert_eq!(next, sample1, "test_binarySearch::2");

        let (previous, next) = oracle.binary_search(storage, 3, 1, 3).unwrap();
        assert_eq!(previous, sample1, "test_binarySearch::3");
        assert_eq!(next, sample2, "test_binarySearch::4");

        let (previous, next) = oracle.binary_search(storage, 3, 9, 3).unwrap();
        assert_eq!(previous, sample1, "test_binarySearch::5");
        assert_eq!(next, sample2, "test_binarySearch::6");

        let (previous, next) = oracle.binary_search(storage, 3, 10, 3).unwrap();
        assert_eq!(previous, sample2, "test_binarySearch::7");
        assert_eq!(next, sample2, "test_binarySearch::8");

        let (previous, next) = oracle.binary_search(storage, 3, 11, 3).unwrap();
        assert_eq!(previous, sample2, "test_binarySearch::9");
        assert_eq!(next, sample3, "test_binarySearch::10");

        let (previous, next) = oracle.binary_search(storage, 3, 20, 3).unwrap();
        assert_eq!(previous, sample3, "test_binarySearch::11");
        assert_eq!(next, sample3, "test_binarySearch::12");
    }

    #[test]
    fn test_binary_search_circular() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        let sample1 = OracleSample::encode(3, 1, 2, 3, 3, 30);
        let sample2 = OracleSample::encode(3, 2, 3, 4, 9, 10);
        let sample3 = OracleSample::encode(3, 3, 4, 5, 9, 20);

        oracle.set_sample(storage, 1, sample1).unwrap();
        oracle.set_sample(storage, 2, sample2).unwrap();
        oracle.set_sample(storage, 3, sample3).unwrap();

        let (previous, next) = oracle.binary_search(storage, 1, 19, 3).unwrap();
        assert_eq!(previous, sample2, "test_binarySearch::1");
        assert_eq!(next, sample2, "test_binarySearch::2");

        let (previous, next) = oracle.binary_search(storage, 1, 24, 3).unwrap();
        assert_eq!(previous, sample2, "test_binarySearch::3");
        assert_eq!(next, sample3, "test_binarySearch::4");

        let (previous, next) = oracle.binary_search(storage, 1, 29, 3).unwrap();
        assert_eq!(previous, sample3, "test_binarySearch::5");
        assert_eq!(next, sample3, "test_binarySearch::6");

        let (previous, next) = oracle.binary_search(storage, 1, 30, 3).unwrap();
        assert_eq!(previous, sample3, "test_binarySearch::7");
        assert_eq!(next, sample1, "test_binarySearch::8");

        let (previous, next) = oracle.binary_search(storage, 1, 33, 3).unwrap();
        assert_eq!(previous, sample1, "test_binarySearch::9");
        assert_eq!(next, sample1, "test_binarySearch::10");
    }

    #[test]
    #[should_panic]
    fn test_revert_binary_search() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        let sample1 = OracleSample::encode(3, 1, 2, 3, 0, 30);
        let sample2 = OracleSample::encode(3, 2, 3, 4, 5, 10);

        // Invalid oracleId
        assert_eq!(
            oracle.binary_search(storage, 0, 20, 3),
            Err(StdError::GenericErr {
                msg: OracleError::InvalidOracleId.to_string()
            }),
            "test_revert_BinarySearch::1"
        );

        // TODO: there is no invalid length error variant
        // Invalid length
        assert_eq!(
            oracle.binary_search(storage, 1, 20, 0),
            Err(StdError::GenericErr {
                msg: OracleError::InvalidOracleId.to_string()
            }),
            "test_revert_BinarySearch::2"
        );

        oracle.set_sample(storage, 1, sample1).unwrap();
        oracle.set_sample(storage, 2, sample2).unwrap();

        // Invalid oracleId
        assert_eq!(
            oracle.binary_search(storage, 0, 20, 3),
            Err(StdError::GenericErr {
                msg: OracleError::InvalidOracleId.to_string()
            }),
            "test_revert_BinarySearch::3"
        );

        // TODO: there is no invalid length error variant
        // Invalid length
        assert_eq!(
            oracle.binary_search(storage, 1, 20, 0),
            Err(StdError::GenericErr {
                msg: OracleError::InvalidOracleId.to_string()
            }),
            "test_revert_BinarySearch::4"
        );

        // Invalid timestamp
        assert_eq!(
            oracle.binary_search(storage, 1, 9, 2),
            Err(StdError::GenericErr {
                msg: OracleError::LookUpTimestampTooOld.to_string()
            }),
            "test_revert_BinarySearch::5"
        );

        // Invalid timestamp
        assert_eq!(
            oracle.binary_search(storage, 1, 31, 2),
            Err(StdError::GenericErr {
                msg: OracleError::LookUpTimestampTooOld.to_string()
            }),
            "test_revert_BinarySearch::6"
        );
    }

    #[test]
    fn test_binary_search_simple_edge_cases() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        // 1. Minimum Length
        let sample_min = OracleSample::encode(1, 1, 2, 3, 0, 0);
        oracle.set_sample(storage, 1, sample_min).unwrap();

        let (previous, next) = oracle.binary_search(storage, 1, 0, 1).unwrap();
        assert_eq!(
            previous, sample_min,
            "test_binary_search_simple_edge_cases::MinLength1"
        );
        assert_eq!(
            next, sample_min,
            "test_binary_search_simple_edge_cases::MinLength2"
        );

        // 2. Maximum Timestamp
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        // FIXME: needs to be u40::MAX instead
        let max_timestamp: u64 = u64::MAX;
        let sample_max = OracleSample::encode(u16::MAX, 1, 2, 3, 0, 0);
        oracle
            .set_sample(storage, u16::MAX - 2, sample_max)
            .unwrap();
        oracle
            .set_sample(storage, u16::MAX - 1, sample_max)
            .unwrap();
        oracle.set_sample(storage, u16::MAX, sample_max).unwrap();

        let (previous, next) = oracle
            .binary_search(storage, u16::MAX - 1, max_timestamp, u16::MAX)
            .unwrap();
        assert_eq!(
            previous, sample_max,
            "test_binary_search_simple_edge_cases::MaxTimestamp1"
        );
        assert_eq!(
            next, sample_max,
            "test_binary_search_simple_edge_cases::MaxTimestamp2"
        );

        // 3. Minimum Timestamp
        let min_timestamp: u64 = 0;
        let sample_min_ts = OracleSample::encode(2, 1, 2, 3, 0, 0);
        oracle.set_sample(storage, 1, sample_min_ts).unwrap();
        oracle.set_sample(storage, 2, sample_min_ts).unwrap();

        let (previous, next) = oracle.binary_search(storage, 1, min_timestamp, 2).unwrap();
        assert_eq!(
            previous, sample_min_ts,
            "test_binary_search_simple_edge_cases::MinTimestamp1"
        );
        assert_eq!(
            next, sample_min_ts,
            "test_binary_search_simple_edge_cases::MinTimestamp2"
        );
    }

    #[test]
    fn test_get_sample_at_fully_initialized() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        let sample1 = OracleSample::encode(3, 40, 50, 60, 3, 30); // sample at timestamp 0 got overriden
        let sample2 = OracleSample::encode(3, 20, 30, 40, 5, 10);
        let sample3 = OracleSample::encode(3, 30, 40, 50, 5, 20);

        oracle.set_sample(storage, 1, sample1).unwrap();
        oracle.set_sample(storage, 2, sample2).unwrap();
        oracle.set_sample(storage, 3, sample3).unwrap();

        let (last_update, cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
            oracle.get_sample_at(storage, 1, 15).unwrap();

        assert_eq!(last_update, 15, "test_GetSampleAt::1");
        assert_eq!(cumulative_id, 20, "test_GetSampleAt::2");
        assert_eq!(cumulative_volatility, 30, "test_GetSampleAt::3");
        assert_eq!(cumulative_bin_crossed, 40, "test_GetSampleAt::4");

        let (last_update, cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
            oracle.get_sample_at(storage, 1, 20).unwrap();

        assert_eq!(last_update, 20, "test_GetSampleAt::5");
        assert_eq!(cumulative_id, 25, "test_GetSampleAt::6");
        assert_eq!(cumulative_volatility, 35, "test_GetSampleAt::7");
        assert_eq!(cumulative_bin_crossed, 45, "test_GetSampleAt::8");

        let (last_update, cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
            oracle.get_sample_at(storage, 1, 25).unwrap();

        assert_eq!(last_update, 25, "test_GetSampleAt::9");
        assert_eq!(cumulative_id, 30, "test_GetSampleAt::10");
        assert_eq!(cumulative_volatility, 40, "test_GetSampleAt::11");
        assert_eq!(cumulative_bin_crossed, 50, "test_GetSampleAt::12");

        let (last_update, cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
            oracle.get_sample_at(storage, 1, 30).unwrap();

        assert_eq!(last_update, 30, "test_GetSampleAt::13");
        assert_eq!(cumulative_id, 36, "test_GetSampleAt::14");
        assert_eq!(cumulative_volatility, 46, "test_GetSampleAt::15");
        assert_eq!(cumulative_bin_crossed, 56, "test_GetSampleAt::16");

        let (last_update, cumulative_id, cumulative_volatility, cumulative_bin_crossed) =
            oracle.get_sample_at(storage, 1, 40).unwrap();

        assert_eq!(last_update, 33, "test_GetSampleAt::17");
        assert_eq!(cumulative_id, 40, "test_GetSampleAt::18");
        assert_eq!(cumulative_volatility, 50, "test_GetSampleAt::19");
        assert_eq!(cumulative_bin_crossed, 60, "test_GetSampleAt::20");
    }

    struct UpdateInputs {
        pub oracle_length: u16,
        pub oracle_id: u16,
        pub previous_active_id: u32, // u24 is not a native Rust type, so we use u32
        pub active_id: u32,          // u24 is not a native Rust type, so we use u32
        pub previous_volatility: u32, // u24 is not a native Rust type, so we use u32
        pub volatility: u32,         // u24 is not a native Rust type, so we use u32
        pub previous_bin_crossed: u32, // u24 is not a native Rust type, so we use u32
        pub created_at: u64,         // u40 is not a native Rust type, so we use u64
        pub timestamp: u64,          // u40 is not a native Rust type, so we use u64
    }

    #[test]
    fn test_update_delta_ts_lower_than_2_minutes() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        // Populate inputs struct (you may want to fuzz these values)
        let mut inputs = UpdateInputs {
            oracle_length: 3,
            oracle_id: 2,
            previous_active_id: 100,
            active_id: 200,
            previous_volatility: 50,
            volatility: 60,
            previous_bin_crossed: 1,
            created_at: 10,
            timestamp: 20,
        };

        inputs.oracle_id = bound(inputs.oracle_id, 1, u16::MAX);
        inputs.oracle_length = bound(inputs.oracle_length, inputs.oracle_id, u16::MAX);
        inputs.created_at = bound(
            inputs.created_at,
            if inputs.timestamp > 120 {
                inputs.timestamp - 120
            } else {
                0
            },
            inputs.timestamp,
        );
        inputs.volatility = bound(inputs.volatility, 1, MASK_UINT20.as_u32());
        inputs.previous_volatility = bound(inputs.previous_volatility, 1, MASK_UINT20.as_u32());

        let sample = OracleSample::encode(
            inputs.oracle_length,
            inputs.previous_active_id as u64 * inputs.created_at,
            inputs.previous_volatility as u64 * inputs.created_at,
            inputs.previous_bin_crossed as u64 * inputs.created_at,
            0,
            inputs.created_at,
        );

        oracle
            .set_sample(storage, inputs.oracle_id, sample)
            .unwrap();

        let mut parameters = PairParameters([0u8; 32]);

        parameters.set_oracle_id(inputs.oracle_id);
        parameters.set_active_id(inputs.previous_active_id).unwrap();
        parameters
            .set_volatility_accumulator(inputs.volatility)
            .unwrap();

        let new_params = oracle
            .update_oracle(storage, inputs.timestamp, parameters, inputs.active_id)
            .unwrap();

        assert_eq!(new_params, parameters, "test_Update::1");

        let sample = oracle.get_sample(storage, inputs.oracle_id).unwrap();

        let dt = inputs.timestamp - inputs.created_at;

        let d_id = if inputs.active_id > inputs.previous_active_id {
            inputs.active_id - inputs.previous_active_id
        } else {
            inputs.previous_active_id - inputs.active_id
        } as u64;

        let cumulative_id = (inputs.previous_active_id as u64 * inputs.created_at)
            + (inputs.previous_active_id as u64 * dt);
        let cumulative_volatility = (inputs.previous_volatility as u64 * inputs.created_at)
            + (inputs.volatility as u64 * dt);
        let cumulative_bin_crossed =
            (inputs.previous_bin_crossed as u64 * inputs.created_at) + (d_id * dt);

        assert_eq!(
            sample.get_oracle_length(),
            inputs.oracle_length,
            "test_Update::3"
        );
        assert_eq!(sample.get_cumulative_id(), cumulative_id, "test_Update::4");
        assert_eq!(
            sample.get_cumulative_volatility(),
            cumulative_volatility,
            "test_Update::5"
        );
        assert_eq!(
            sample.get_cumulative_bin_crossed(),
            cumulative_bin_crossed,
            "test_Update::6"
        );
    }

    #[test]
    fn test_update_delta_ts_greater_than_2_minutes() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        // Populate inputs struct (you may want to fuzz these values)
        let inputs = UpdateInputs {
            oracle_length: 3,
            oracle_id: 2,
            previous_active_id: 100,
            active_id: 200,
            previous_volatility: 50,
            volatility: 60,
            previous_bin_crossed: 1,
            created_at: 10,
            timestamp: 140,
        };

        assert!(
            inputs.oracle_id > 0
                && inputs.oracle_length >= inputs.oracle_id
                && inputs.created_at <= inputs.timestamp
                && inputs.timestamp - inputs.created_at > 120
                && inputs.volatility <= MASK_UINT20.as_u32()
                && inputs.previous_volatility <= MASK_UINT20.as_u32()
        );

        let sample = OracleSample::encode(
            inputs.oracle_length,
            inputs.previous_active_id as u64 * inputs.created_at,
            inputs.previous_volatility as u64 * inputs.created_at,
            inputs.previous_bin_crossed as u64 * inputs.created_at,
            0,
            inputs.created_at,
        );

        oracle
            .set_sample(storage, inputs.oracle_id, sample)
            .unwrap();

        let mut parameters = PairParameters([0u8; 32]);

        parameters.set_oracle_id(inputs.oracle_id);
        parameters.set_active_id(inputs.previous_active_id).unwrap();
        parameters
            .set_volatility_accumulator(inputs.volatility)
            .unwrap();

        let mut new_params = oracle
            .update_oracle(storage, inputs.timestamp, parameters, inputs.active_id)
            .unwrap();

        let next_id = ((inputs.oracle_id as usize % inputs.oracle_length as usize) + 1) as u16;

        assert_eq!(
            new_params.set_oracle_id(next_id).clone(),
            new_params,
            "test_Update::1"
        );

        if inputs.oracle_length > 1 {
            assert_eq!(
                oracle.get_sample(storage, inputs.oracle_id).unwrap(),
                sample,
                "test_Update::2"
            );
        }

        let sample = oracle.get_sample(storage, next_id).unwrap();

        let dt = inputs.timestamp - inputs.created_at;

        let d_id = if inputs.active_id > inputs.previous_active_id {
            inputs.active_id - inputs.previous_active_id
        } else {
            inputs.previous_active_id - inputs.active_id
        } as u64;

        let cumulative_id = (inputs.previous_active_id as u64 * inputs.created_at)
            + (inputs.previous_active_id as u64 * dt);
        let cumulative_volatility = (inputs.previous_volatility as u64 * inputs.created_at)
            + (inputs.volatility as u64 * dt);
        let cumulative_bin_crossed =
            (inputs.previous_bin_crossed as u64 * inputs.created_at) + (d_id * dt);

        assert_eq!(
            sample.get_oracle_length(),
            inputs.oracle_length,
            "test_Update::3"
        );
        assert_eq!(sample.get_cumulative_id(), cumulative_id, "test_Update::4");
        assert_eq!(
            sample.get_cumulative_volatility(),
            cumulative_volatility,
            "test_Update::5"
        );
        assert_eq!(
            sample.get_cumulative_bin_crossed(),
            cumulative_bin_crossed,
            "test_Update::6"
        );
    }

    #[test]
    fn test_increase_oracle_length() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        // Random lengths, you may want to fuzz these values.
        let length = 3;
        let new_length = 4;

        let oracle_id = 1;

        oracle.increase_length(storage, oracle_id, length).unwrap();

        println!(
            "{:#?}",
            oracle
                .get_sample(storage, oracle_id)
                .unwrap()
                .get_oracle_length()
        );

        oracle
            .increase_length(storage, oracle_id, new_length)
            .unwrap();

        println!(
            "{:#?}",
            oracle
                .get_sample(storage, oracle_id)
                .unwrap()
                .get_oracle_length()
        );

        assert_eq!(
            oracle
                .get_sample(storage, oracle_id)
                .unwrap()
                .get_oracle_length(),
            new_length,
            "test_IncreaseOracleLength::1"
        );

        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        // Random lengths, you may want to fuzz these values.
        let length = u16::MAX - 1;
        let new_length = u16::MAX;

        let oracle_id = 1;

        oracle.increase_length(storage, oracle_id, length).unwrap();
        oracle
            .increase_length(storage, oracle_id, new_length)
            .unwrap();

        assert_eq!(
            oracle
                .get_sample(storage, oracle_id)
                .unwrap()
                .get_oracle_length(),
            new_length,
            "test_IncreaseOracleLength::2"
        );
    }

    #[test]
    fn test_revert_increase_oracle_length() {
        let mut deps = mock_dependencies();
        let storage = &mut deps.storage;
        let oracle: Map<u16, OracleSample> = Map::new("oracle");

        // Random lengths, you may want to fuzz these values.
        let length = 3;
        let new_length = 2;

        // Equivalent to vm.assume in Solidity
        assert!(new_length <= length && length > 0);

        let oracle_id = 1;

        oracle.increase_length(storage, oracle_id, length).unwrap();

        // Equivalent to vm.expectRevert in Solidity.
        // Replace with your own logic.
        assert!(oracle
            .increase_length(storage, oracle_id, new_length)
            .is_err());
    }
}
