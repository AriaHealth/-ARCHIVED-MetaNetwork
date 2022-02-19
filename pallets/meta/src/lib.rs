#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_support::{
        sp_runtime::traits::Hash,
        traits::{tokens::ExistenceRequirement, Currency},
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
    pub struct Meta<T: Config> {
        pub sender: AccountOf<T>,
        pub metahash: Vec<u8>,
        pub owner: Vec<u8>,
        pub price: Option<CoinOf<T>>,
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct VirtualAccount<T: Config> {
        pub owner: Vec<u8>,
        pub beneficiary: Option<Vec<u8>>,
        pub sender: AccountOf<T>,
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Cheque<T: Config> {
        pub owner: Vec<u8>,
        pub claimed: bool,
        pub amount: CoinOf<T>,
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
        type MaxMetasCreated: Get<u32>;

        #[pallet::constant]
        type MaxVirtualAccountsOwned: Get<u32>;
    }

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        MetaCountOverflow,
        VirtualAccountCountOverflow,
        ExceedMaxMetasOwned,
        ExceedMaxMetasCreated,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        MetaCreated(T::AccountId, T::Hash),
        VirtualAccountCreated(Vec<u8>, Vec<u8>),
    }

    #[pallet::storage]
    #[pallet::getter(fn metas_count)]
    pub(super) type MetasCount<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn escrow_coin)]
    pub(super) type EscrowCoin<T: Config> = StorageValue<_, CoinOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn metas)]
    pub(super) type Metas<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Meta<T>>;

    #[pallet::storage]
    #[pallet::getter(fn metas_owned)]
    pub(super) type MetasOwned<T: Config> =
        StorageMap<_, Twox64Concat, Vec<u8>, BoundedVec<T::Hash, T::MaxMetasOwned>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn metas_created)]
    pub(super) type MetasCreated<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        BoundedVec<T::Hash, T::MaxMetasCreated>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn virtual_account_count)]
    pub(super) type VirtualAccountCount<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn virtual_accounts)]
    pub(super) type VirtualAccounts<T: Config> =
        StorageMap<_, Twox64Concat, Vec<u8>, VirtualAccount<T>>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100)]
        pub fn create_meta(
            origin: OriginFor<T>,
            metahash: Vec<u8>,
            owner: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let meta = Meta::<T> {
                sender: sender.clone(),
                metahash: metahash.clone(),
                owner: owner.clone(),
                price: None,
            };

            let meta_id = T::Hashing::hash_of(&meta);

            let new_cnt = Self::metas_count()
                .checked_add(1)
                .ok_or(<Error<T>>::MetaCountOverflow)?;

            <MetasOwned<T>>::try_mutate(&owner, |meta_vec| meta_vec.try_push(meta_id))
                .map_err(|_| <Error<T>>::ExceedMaxMetasOwned)?;
            <MetasCreated<T>>::try_mutate(&sender, |meta_vec| meta_vec.try_push(meta_id))
                .map_err(|_| <Error<T>>::ExceedMaxMetasCreated)?;

            <Metas<T>>::insert(meta_id, meta);
            <MetasCount<T>>::put(new_cnt);

            log::info!("A meta medical record is minted with ID: {:?}.", meta_id);
            Self::deposit_event(Event::MetaCreated(sender, meta_id));

            Ok(())
        }

        #[pallet::weight(100)]
        pub fn create_virtual_account(
            origin: OriginFor<T>,
            virtual_account_id: Vec<u8>,
            owner: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let virtual_account = VirtualAccount {
                owner: owner.clone(),
                sender: sender.clone(),
                beneficiary: None,
            };

            // Performs this operation first as it may fail
            let new_cnt = Self::virtual_account_count()
                .checked_add(1)
                .ok_or(<Error<T>>::VirtualAccountCountOverflow)?;

            <VirtualAccounts<T>>::insert(virtual_account_id.clone(), virtual_account);
            <VirtualAccountCount<T>>::put(new_cnt);

            log::info!(
                "A virtual account is created with ID: {:?}.",
                &virtual_account_id
            );
            Self::deposit_event(Event::VirtualAccountCreated(virtual_account_id, owner));

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {}
}
