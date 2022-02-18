#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_support::{
        sp_runtime::traits::Hash,
        traits::{tokens::ExistenceRequirement, Currency, Randomness},
        transactional,
    };
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_io::hashing::blake2_128;
    use sp_std::vec::Vec;

    #[cfg(feature = "std")]
    use frame_support::serde::{Deserialize, Serialize};

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
        pub hash: Vec<u8>,
        pub owner: AccountOf<T>,
        pub uniq: [u8; 16],
    }

    // Struct for holding virtual wallet
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct VirtualWallet {
        pub beneficiary: Vec<u8>,
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
        RegisterVirtualPatient = 43,
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
        VirtualPatient = 31,
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

        /// The maximum amount of action records a single account can own.
        #[pallet::constant]
        type MaxRecordsOwned: Get<u32>;

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
        ExceedMaxActionOwned,
        VirtualWalletCountOverflow,
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
        VirtualWalletCreated(T::AccountId, Vec<u8>),
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

    // Storage item to keep a count of all existing action records
    #[pallet::storage]
    #[pallet::getter(fn virtual_wallet_count)]
    /// Keeps track of the number of actiton in existence.
    pub(super) type VirtualWalletCount<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn virtual_wallets)]
    /// Stores an virtual wallet record
    pub(super) type VirtualWallets<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, VirtualWallet>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // TODO : submit action_record
        #[pallet::weight(100)]
        pub fn submit_action_record(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // log::info!("🧑‍⚕️ An action record is minted with ID ➡ {:?}.", record_id);
            // Self::deposit_event(Event::RecordSubmitted(sender, record_id));
            Ok(())
        }

        // TODO : submit virtual_wallet
        #[pallet::weight(100)]
        pub fn submit_virtual_wallet(
            origin: OriginFor<T>,
            virtual_wallet_id: T::AccountId,
            beneficiary: Vec<u8>,
        ) -> DispatchResult {
            // // TODO ensure the sender is SUDO
            let sender = ensure_signed(origin)?;

            let kitty_id = Self::register_virtual_wallet(&virtual_wallet_id, &beneficiary);

            log::info!(
                "A virtual wallet is createdis born with ID: {:?}.",
                &virtual_wallet_id
            );
            Self::deposit_event(Event::VirtualWalletCreated(virtual_wallet_id, beneficiary));

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

        pub fn mint(
            action: &ActionType,
            hash: &Vec<u8>,
            owner: &T::AccountId,
            uniq: &[u8; 16],
        ) -> Result<T::Hash, Error<T>> {
            let action_record = ActionRecord::<T> {
                action: action.clone(),
                hash: hash.clone(),
                owner: owner.clone(),
                uniq: uniq.clone(),
            };

            let action_record_id = T::Hashing::hash_of(&action_record);

            // Performs this operation first as it may fail
            let new_cnt = Self::action_count()
                .checked_add(1)
                .ok_or(<Error<T>>::ActionCountOverflow)?;

            // Performs this operation first because as it may fail

            <ActionRecordsOwned<T>>::mutate(&owner, |action_vec| action_vec.push(action_record_id));

            <ActionRecords<T>>::insert(action_record_id, action_record);
            <ActionCount<T>>::put(new_cnt);
            Ok(action_record_id)
        }

        pub fn register_virtual_wallet(
            virtual_wallet_id: &T::AccountId,
            beneficiary: &Vec<u8>,
        ) -> Result<T::AccountId, Error<T>> {
            let virtual_wallet = VirtualWallet {
                beneficiary: beneficiary.clone(),
            };

            // Performs this operation first as it may fail
            let new_cnt = Self::virtual_wallet_count()
                .checked_add(1)
                .ok_or(<Error<T>>::VirtualWalletCountOverflow)?;

            <VirtualWallets<T>>::insert(virtual_wallet_id.clone(), virtual_wallet);
            <VirtualWalletCount<T>>::put(new_cnt);
            Ok(virtual_wallet_id.clone())
        }

        // TODO Part III: helper functions for dispatchable functions

        // TODO: increment_nonce, random_hash, mint, transfer_from
    }
}
