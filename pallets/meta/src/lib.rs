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

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct MetaMedicalRecord<T: Config> {
        pub sender: AccountOf<T>,
        pub hash: Vec<u8>,
        pub owner: AccountOf<T>,
        pub uniq: [u8; 16],
    }

    // Struct for holding virtual account
    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct VirtualAccount<T: Config> {
        pub beneficiary: Vec<u8>,
        pub sender: AccountOf<T>,
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub enum ResourceType {
        Observation,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types it depends on.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency handler
        type Coin: Currency<Self::AccountId>;

        #[pallet::constant]
        type MaxMetasOwned: Get<u32>;

        #[pallet::constant]
        type MaxVirtualAccountsOwned: Get<u32>;

        /// The randomness property of actions pallet.
        type Uniqueness: Randomness<Self::Hash, Self::BlockNumber>;
    }

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        MetaMedicalRecordCountOverflow,
        VirtualAccountCountOverflow,
        ExceedMaxMetaMedicalRecordOwned,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        MetaMedicalRecord(T::AccountId, T::Hash),
        VirtualAccountCreated(T::AccountId, T::AccountId),
    }

    #[pallet::storage]
    #[pallet::getter(fn action_count)]
    pub(super) type MetaMedicalRecordsCount<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn action_records)]
    pub(super) type MetaMedicalRecords<T: Config> =
        StorageMap<_, Twox64Concat, T::Hash, MetaMedicalRecord<T>>;

    #[pallet::storage]
    #[pallet::getter(fn action_records_owned)]
    pub(super) type MetaMedicalRecordsOwned<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        BoundedVec<T::Hash, T::MaxMetasOwned>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn virtual_account_count)]
    pub(super) type VirtualAccountCount<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn virtual_accounts)]
    pub(super) type VirtualAccounts<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, VirtualAccount<T>>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100)]
        pub fn submit_meta_medical_record(
            origin: OriginFor<T>,
            hash: Vec<u8>,
            owner: T::AccountId,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let meta_medical_record_id = Self::mint(&sender, &hash, &owner)?;

            log::info!(
                "A meta medical record is minted with ID: {:?}.",
                meta_medical_record_id
            );
            Self::deposit_event(Event::MetaMedicalRecord(sender, meta_medical_record_id));

            Ok(())
        }

        #[pallet::weight(100)]
        pub fn submit_virtual_account(
            origin: OriginFor<T>,
            virtual_account_id: T::AccountId,
            beneficiary: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let _ = Self::register(&sender, &virtual_account_id, &beneficiary);

            log::info!(
                "A virtual account is created with ID: {:?}.",
                &virtual_account_id
            );
            Self::deposit_event(Event::VirtualAccountCreated(virtual_account_id, sender));

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn generate_uniqueness() -> [u8; 16] {
            let payload = (
                T::Uniqueness::random(&b"uniq"[..]).0,
                <frame_system::Pallet<T>>::block_number(),
            );
            payload.using_encoded(blake2_128)
        }

        pub fn mint(
            sender: &T::AccountId,
            hash: &Vec<u8>,
            owner: &T::AccountId,
        ) -> Result<T::Hash, Error<T>> {
            let action_record = MetaMedicalRecord::<T> {
                sender: sender.clone(),
                hash: hash.clone(),
                owner: owner.clone(),
                uniq: Self::generate_uniqueness(),
            };

            let meta_medical_record_id = T::Hashing::hash_of(&action_record);

            let new_cnt = Self::action_count()
                .checked_add(1)
                .ok_or(<Error<T>>::MetaMedicalRecordCountOverflow)?;

            <MetaMedicalRecordsOwned<T>>::try_mutate(&owner, |meta_medical_record_vec| {
                meta_medical_record_vec.try_push(meta_medical_record_id)
            })
            .map_err(|_| <Error<T>>::ExceedMaxMetaMedicalRecordOwned)?;

            <MetaMedicalRecords<T>>::insert(meta_medical_record_id, action_record);
            <MetaMedicalRecordsCount<T>>::put(new_cnt);
            Ok(meta_medical_record_id)
        }

        pub fn register(
            sender: &T::AccountId,
            virtual_account_id: &T::AccountId,
            beneficiary: &Vec<u8>,
        ) -> Result<T::AccountId, Error<T>> {
            let virtual_account = VirtualAccount {
                beneficiary: beneficiary.clone(),
                sender: sender.clone(),
            };

            // Performs this operation first as it may fail
            let new_cnt = Self::virtual_account_count()
                .checked_add(1)
                .ok_or(<Error<T>>::VirtualAccountCountOverflow)?;

            <VirtualAccounts<T>>::insert(virtual_account_id.clone(), virtual_account);
            <VirtualAccountCount<T>>::put(new_cnt);
            Ok(virtual_account_id.clone())
        }
    }
}
