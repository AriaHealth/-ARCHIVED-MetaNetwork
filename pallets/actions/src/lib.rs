#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo},
        pallet_prelude::*,
        sp_runtime::traits::{Hash, Zero},
        traits::{Currency, ExistenceRequirement, Randomness},
    };
    use frame_system::pallet_prelude::*;
    use sp_io::hashing::blake2_128;
    use sp_std::vec::Vec;

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
        pub uniq: [u8; 16],
    }

    // Set ActionType
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    // EACH ACTION TYPE MUST HAVE THEIR OWN DISPATCHABLE
    pub enum ActionType {
        SubmitRecord = 0,
        AmendRecord = 1,
        TransferRecord = 2,
        ShareRecord = 3,
        AuctionRecord = 4,
        BuyRecord = 5,
        DestroyRecord = 6,
        CrowdsourceCollection = 20,
        CreateCollection = 21,
        JoinCollection = 22,
        RegisterActor = 40,
        UpdateActor = 41,
        DestroyActor = 42,
        DepositCoin = 60,
        WithdrawCoin = 61,
        TransferCoin = 62,
    }

    // Set ActorRole
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Copy)]
    #[scale_info(skip_type_params(T))]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub enum ActorRole {
        MedicalCenter = 10,
        MedicalProfessional = 20,
        Patient = 30,
        Aggregator = 40,
        Observer = 50,
        Node = 60,
        Sudoer = 70,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types it depends on.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency handler for the actions pallet.
        type Coin: Currency<Self::AccountId>;
        type Token: Currency<Self::AccountId>;

        /// The randomness property of actions pallet.
        type Uniqueness: Randomness<Self::Hash, Self::BlockNumber>;
    }

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        ActionCountOverflow,
        ActorCountOverflow,
        ActorDestroyIsNotAllowed,
        AuctionPriceTooHigh,
        AuctionPriceTooLow,
        BuyerAlreadyOwnedRecordAccess,
        BuyerIsRecordOwner,
        CrowdsourceQuotaReachLimit,
        InsufficientActorRight,
        NotEnoughCoin,
        NotEnoughToken,
        NotRecordOwner,
        RecordDestroyNotAllowed,
        RecordNotExist,
        RecordNotForSale,
        ShareToSelf,
        TransferToSelf,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        RecordSubmitted(T::AccountId, T::Hash),
        RecordAmended(T::AccountId, T::Hash),
        RecordTransferred(T::AccountId, T::Hash),
        RecordShared(T::AccountId, T::Hash),
        RecordAuctioned(T::AccountId, T::Hash),
        RecordBought(T::AccountId, T::Hash),
        RecordDestroyed(T::AccountId, T::Hash),
        CollectionCrowdsourced(T::AccountId, T::Hash),
        CollectionCreated(T::AccountId, T::Hash),
        RecordJoinedCollection(T::AccountId, T::Hash),
        ActorRegistered(T::AccountId, T::Hash),
        ActorUpdated(T::AccountId, T::Hash),
        ActorDestroyed(T::AccountId, T::Hash),
        CoinDeposited(T::AccountId, T::Hash),
        CoinWithdrawn(T::AccountId, T::Hash),
        CoinTransferred(T::AccountId, T::Hash),
    }

    // Storage item to keep a count of all existing action records
    #[pallet::storage]
    #[pallet::getter(fn action_count)]
    /// Keeps track of the number of actiton in existence.
    pub(super) type ActionCount<T: Config> = StorageValue<_, u128, ValueQuery>;

    // Storage item to keep all action records
    #[pallet::storage]
    #[pallet::getter(fn action_records)]
    /// Stores an action record
    pub(super) type ActionRecords<T: Config> =
        StorageMap<_, Twox64Concat, T::Hash, ActionRecord<T>>;

    // Storage item to keep all action records ownership
    #[pallet::storage]
    #[pallet::getter(fn action_records_owned)]
    /// Keeps track of what accounts own what action record.
    pub(super) type ActionRecordsOwned<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, Vec<T::Hash>, ValueQuery>;

    // Storage item to keep all actor in existence
    #[pallet::storage]
    #[pallet::getter(fn actor_count)]
    /// Keeps track of the number of actor in existence.
    pub(super) type ActorCount<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn actor_whois)]
    /// Keeps track of what accounts own what role.
    pub(super) type ActorWhois<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, u8, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO : submit_record
        #[pallet::weight(100)]
        pub fn submit_record(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::RecordSubmitted(sender, record_id));
            Ok(())
        }

        // TODO : amend_record
        #[pallet::weight(100)]
        pub fn amend_record(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::RecordAmended(sender, record_id));
            Ok(())
        }

        // TODO : transfer_record
        #[pallet::weight(100)]
        pub fn transfer_record(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::RecordTransferred(sender, record_id));
            Ok(())
        }

        // TODO : share_record
        #[pallet::weight(100)]
        pub fn share_record(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::RecordShared(sender, record_id));
            Ok(())
        }

        // TODO : auction_record
        #[pallet::weight(100)]
        pub fn auction_record(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::RecordAuctioned(sender, record_id));
            Ok(())
        }

        // TODO : buy_record
        #[pallet::weight(100)]
        pub fn buy_record(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::RecordBought(sender, record_id));
            Ok(())
        }

        // TODO : destroy_record
        #[pallet::weight(100)]
        pub fn destroy_record(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::RecordDestroyed(sender, record_id));
            Ok(())
        }

        // TODO : crowdsource_collection
        #[pallet::weight(100)]
        pub fn crowdsource_collection(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::CollectionCrowdsourced(sender, record_id));
            Ok(())
        }

        // TODO : create_collection
        #[pallet::weight(100)]
        pub fn create_collection(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::CollectionCreated(sender, record_id));
            Ok(())
        }

        // TODO : join_collection
        #[pallet::weight(100)]
        pub fn join_collection(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::RecordJoinedCollection(sender, record_id));
            Ok(())
        }

        // TODO : register_actor
        #[pallet::weight(100)]
        pub fn register_actor(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::ActorRegistered(sender, record_id));
            Ok(())
        }

        // TODO : update_actor
        #[pallet::weight(100)]
        pub fn update_actor(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::ActorUpdated(sender, record_id));
            Ok(())
        }

        // TODO : destroy_actor
        #[pallet::weight(100)]
        pub fn destroy_actor(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::ActorDestroyed(sender, record_id));
            Ok(())
        }

        // TODO : deposit_coin
        #[pallet::weight(100)]
        pub fn deposit_coin(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::CoinDeposited(sender, record_id));
            Ok(())
        }

        // TODO : withdraw_coin
        #[pallet::weight(100)]
        pub fn withdraw_coin(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::CoinWithdrawn(sender, record_id));
            Ok(())
        }

        // TODO : transfer_coin
        #[pallet::weight(100)]
        pub fn transfer_coin(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::CoinTransferred(sender, record_id));
            Ok(())
        }
    }

    // TODO Part II: helper function for actions struct

    impl<T: Config> Pallet<T> {
        fn generate_uniqueness() -> [u8; 16] {
            let payload = (
                T::Uniqueness::random(&b"uniq"[..]).0,
                <frame_system::Pallet<T>>::block_number(),
            );
            payload.using_encoded(blake2_128)
        }
        // TODO Part III: helper functions for dispatchable functions

        // TODO: increment_nonce, random_hash, mint, transfer_from
    }
}
