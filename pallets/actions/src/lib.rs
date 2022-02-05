#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use chrono::prelude::*;
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo},
        pallet_prelude::*,
        sp_runtime::traits::{Hash, Zero},
        traits::{Currency, ExistenceRequirement, Randomness},
    };
    use frame_system::pallet_prelude::*;
    use sp_io::hashing::blake2_128;
    use sp_std::vec::Vec;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[cfg(feature = "std")]
    use frame_support::serde::{Deserialize, Serialize};
    use scale_info::TypeInfo;

    type AccountOf<T> = <T as frame_system::Config>::AccountId;
    type CoinOf<T> =
        <<T as Config>::Coin as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type TokenOf<T> =
        <<T as Config>::Token as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // Struct for holding action record
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct ActionRecord<T: Config> {
        pub action: ActionType,
        pub epoch: u128,
        pub hash: Vec<u8>,
        pub owner: AccountOf<T>,
        pub ttl: u128,
    }

    // Set ActionType
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    // YOU MAY ADD THE ACTION, BUT DO NOT CHANGE THE ORDER
    pub enum ActionType {
        SubmitRecord,
        AmendRecord,
        TransferRecord,
        ShareRecord,
        AuctionRecord,
        BuyRecord,
        CrowdsourceCollection,
        CreateCollection,
        JoinCollection,
        DestroyRecord,
        RegisterActor,
        UpdateActor,
        DestroyActor,
        DepositCoin,
        WithdrawCoin,
        TransferCoin,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types it depends on.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The Currency handler for the Kitties pallet.
        type Coin: Currency<Self::AccountId>;
        type Token: Currency<Self::AccountId>;

        // TODO Part II: Specify the custom types for our runtime.
    }

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        // TODO Part III
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // TODO Part III
    }

    // Storage item to keep a count of all existing action records
    #[pallet::storage]
    #[pallet::getter(fn action_cnt)]
    /// Keeps track of the number of actiton in existence.
    pub(super) type ActionCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

    // TODO Part II: Remaining storage items.

    // TODO Part III: Our pallet's genesis configuration.

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO Part III: create_kitty

        // TODO Part III: set_price

        // TODO Part III: transfer

        // TODO Part III: buy_kitty

        // TODO Part III: breed_kitty
    }

    // TODO Part II: helper function for Kitty struct

    impl<T: Config> Pallet<T> {
        // TODO Part III: helper functions for dispatchable functions

        // TODO: increment_nonce, random_hash, mint, transfer_from
    }
}
