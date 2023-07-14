// This file is part of Substrate.

// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Various pieces of common functionality.
use super::*;
use log::{info};
use frame_support::{
	traits::{Get, StorageVersion, GetStorageVersion},
	weights::Weight, storage_alias, Twox64Concat, BoundedVec
};
const LOG_TARGET: &str = "nicks";


// only contains V1 storage format
pub mod v1 {
    use super::*;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

    #[storage_alias]
	pub(super) type NameOf<T: Config> =
		StorageMap<Pallet<T>, Twox64Concat, <T as frame_system::Config>::AccountId, (BoundedVec<u8, <T as pallet::Config>::MaxLength>, BalanceOf<T>)>;
} 


// contains checks and transforms storage to V2 format
pub fn migrate_to_v2<T: Config>() -> Weight {
    let onchain_version =  Pallet::<T>::on_chain_storage_version();
    if onchain_version < 2 {
            // migrate to v2
            // Very inefficient, mostly here for illustration purposes.
			let count = v1::NameOf::<T>::iter().count();
			info!(target: LOG_TARGET, " >>> Updating MyNicks storage. Migrating {} nicknames...", count);

			// We transform the storage values from the old into the new format.
			NameOf::<T>::translate::<(Vec<u8>, BalanceOf<T>), _>(
				|k: T::AccountId, (nick, deposit): (Vec<u8>, BalanceOf<T>)| {
					info!(target: LOG_TARGET, "     Migrated nickname for {:?}...", k);

					// We split the nick at ' ' (<space>).
					match nick.iter().rposition(|&x| x == b" "[0]) {
						Some(ndx) => {
                            let bounded_first: BoundedVec<_, _> = nick[0..ndx].to_vec().try_into().unwrap();
                            let bounded_last: BoundedVec<_, _> = nick[ndx + 1..].to_vec().try_into().unwrap();
                            Some((Nickname {
							    first: bounded_first,
							    last: Some(bounded_last)
						    }, deposit))
                    },
						None => {
                            let bounded_name: BoundedVec<_, _> = nick.to_vec().try_into().unwrap();
                            Some((Nickname { first: bounded_name, last: None }, deposit))
                        }
					}
				}
			);

			// Update storage version.
			StorageVersion::new(2).put::<Pallet::<T>>();
			// Very inefficient, mostly here for illustration purposes.
			let count = NameOf::<T>::iter().count();
			info!(target: LOG_TARGET," <<< MyNicks storage updated! Migrated {} nicknames ✅", count);
			// Return the weight consumed by the migration.
			T::DbWeight::get().reads_writes(count as u64 + 1, count as u64 + 1)
    
    } else {
        info!(target: LOG_TARGET, " >>> Unused migration!");
        // We don't do anything here.
		Weight::zero()
    }

} 