#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use argon2::{
        password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
        Argon2,
    };
    use frame_support::pallet_prelude::*;
    use frame_support::{
        sp_runtime::traits::Hash,
        traits::{tokens::ExistenceRequirement, Currency, ReservableCurrency},
        transactional,
    };
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_io::hashing::blake2_128;
    use sp_std::vec::Vec;

    #[cfg(feature = "std")]
    use frame_support::serde::{Deserialize, Serialize};

    /// Configure the pallet by specifying the parameters and types it depends on.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency handler
        type Coin: ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type MaxMetasOwned: Get<u32>;

        #[pallet::constant]
        type MaxMetasGranted: Get<u32>;
    }

    type AccountOf<T> = <T as frame_system::Config>::AccountId;
    type CoinOf<T> =
        <<T as Config>::Coin as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct Meta<T: Config> {
        pub creator: AccountOf<T>,    // meta creator
        pub metahash: Vec<u8>,        // meta SHA256 hash
        pub owner: T::Hash,           // meta account id
        pub price: Option<CoinOf<T>>, // price of the meta
        pub metaformat: MetaFormat,
        pub region: Region,
        pub resource: Option<ResourceType>,
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct MetaAccount<T: Config> {
        pub escrow: AccountOf<T>,         // chosen intermediary
        pub creator: AccountOf<T>,        // meta accout creator
        pub credit: u64,                  // meta account balance
        pub beneficiary: Option<Vec<u8>>, // argon2id hashed wallet address
        pub identity: Vec<u8>,            // argon2id hashed owner email
        pub is_virtual: bool,             // false means creator is the owner
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub enum ResourceType {
        Observation,
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub enum MetaFormat {
        FhirR4 = 1,
        FhirSTU = 2,
        FhirDSTU = 3,
        Hl7 = 4,
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub enum Region {
        Europe = 1,
        MiddleEast = 2,
        Africa = 3,
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // Errors.
    #[pallet::error]
    pub enum Error<T> {
        BuyerIsMetaCreator,
        ExceedMaxMetasGranted,
        ExceedMaxMetasOwned,
        MetaAccountCountOverflow,
        MetaAccountNotExist,
        MetaCountOverflow,
        MetaNotExist,
        MetaNotForSale,
        NotEnoughCoin,
        UnprocessableCoin,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        MetaCreated(T::AccountId, T::Hash),
        MetaBought(T::AccountId, T::Hash),
        MetaAccountCreated(T::Hash),
    }

    #[pallet::storage]
    #[pallet::getter(fn metas_count)]
    pub(super) type MetasCount<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn metas)]
    pub(super) type Metas<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Meta<T>>;

    #[pallet::storage]
    #[pallet::getter(fn metas_owned)]
    pub(super) type MetasOwned<T: Config> =
        StorageMap<_, Twox64Concat, T::Hash, BoundedVec<T::Hash, T::MaxMetasOwned>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn metas_granted)]
    pub(super) type MetasGranted<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        BoundedVec<T::Hash, T::MaxMetasGranted>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn meta_account_count)]
    pub(super) type MetaAccountCount<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn meta_accounts)]
    pub(super) type MetaAccounts<T: Config> = StorageMap<_, Twox64Concat, T::Hash, MetaAccount<T>>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_meta(
            origin: OriginFor<T>,
            metahash: Vec<u8>,
            owner: T::Hash,
            price: Option<CoinOf<T>>,
            metaformat: MetaFormat,
            region: Region,
            resource: Option<ResourceType>,
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;

            // TODO ensure the owner hash is exist

            let meta = Meta::<T> {
                creator: creator.clone(),
                metahash: metahash.clone(),
                owner: owner.clone(),
                price: price.clone(),
                metaformat: metaformat.clone(),
                region: region.clone(),
                resource: resource.clone(),
            };

            let meta_id = T::Hashing::hash_of(&meta);

            let new_cnt = Self::metas_count()
                .checked_add(1)
                .ok_or(<Error<T>>::MetaCountOverflow)?;

            <MetasOwned<T>>::try_mutate(&owner, |meta_vec| meta_vec.try_push(meta_id))
                .map_err(|_| <Error<T>>::ExceedMaxMetasOwned)?;
            <MetasGranted<T>>::try_mutate(&creator, |meta_vec| meta_vec.try_push(meta_id))
                .map_err(|_| <Error<T>>::ExceedMaxMetasGranted)?;

            <Metas<T>>::insert(meta_id, meta);
            <MetasCount<T>>::put(new_cnt);

            log::info!("A meta is minted [ID {:?}]", meta_id);
            Self::deposit_event(Event::MetaCreated(creator, meta_id));

            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn buy_meta(origin: OriginFor<T>, meta_id: T::Hash) -> DispatchResult {
            let buyer = ensure_signed(origin)?;

            let meta = Self::metas(&meta_id).ok_or(<Error<T>>::MetaNotExist)?;
            let mut owner =
                Self::meta_accounts(&meta.owner).ok_or(<Error<T>>::MetaAccountNotExist)?;

            // Check if buyer is not the meta creator
            ensure!(meta.creator != buyer, <Error<T>>::BuyerIsMetaCreator);

            // TODO check if buyer is the real owner, if yes throw error. Validate using argon2

            // TODO check if buyer is already buy the meta

            // Grant the buyer meta if it has the capacity to receive one more meta
            <MetasGranted<T>>::try_mutate(&buyer, |meta_vec| meta_vec.try_push(meta_id))
                .map_err(|_| <Error<T>>::ExceedMaxMetasGranted)?;

            let escrow = owner.escrow.clone();

            if let Some(price) = meta.price {
                // Check the buyer has enough free coin
                ensure!(
                    T::Coin::free_balance(&buyer) >= price,
                    <Error<T>>::NotEnoughCoin
                );

                // Transfer the amount from buyer to escrow
                T::Coin::transfer(&buyer, &escrow, price, ExistenceRequirement::KeepAlive)?;
                T::Coin::reserve(&escrow, price)?;

                // Increase owner credit
                if let Some(credit) = Self::coin_to_u64(price) {
                    owner.credit = owner.credit.clone() + credit;
                    <MetaAccounts<T>>::insert(&meta.owner, owner);
                } else {
                    Err(<Error<T>>::UnprocessableCoin)?;
                }
            } else {
                // Check the meta is for sale
                Err(<Error<T>>::MetaNotForSale)?;
            }

            log::info!("A meta is bought [ID {:?}]", meta_id);
            Self::deposit_event(Event::MetaBought(buyer, meta_id));

            Ok(())
        }

        #[pallet::weight(1000)]
        pub fn create_meta_account(
            origin: OriginFor<T>,
            escrow: T::AccountId,
            identity: Vec<u8>,
            is_virtual: bool,
            beneficiary: Option<Vec<u8>>,
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;

            let meta_account = MetaAccount {
                identity: identity.clone(),
                creator: creator.clone(),
                escrow: escrow.clone(),
                beneficiary: beneficiary.clone(),
                credit: 0,
                is_virtual: is_virtual,
            };

            let meta_account_id = T::Hashing::hash_of(&meta_account);

            // Performs this operation first as it may fail
            let new_cnt = Self::meta_account_count()
                .checked_add(1)
                .ok_or(<Error<T>>::MetaAccountCountOverflow)?;

            <MetaAccounts<T>>::insert(meta_account_id.clone(), meta_account);
            <MetaAccountCount<T>>::put(new_cnt);

            log::info!("A meta account is created with ID: {:?}.", &meta_account_id);
            Self::deposit_event(Event::MetaAccountCreated(meta_account_id));

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // from https://stackoverflow.com/questions/56081117
        pub fn u64_to_coin_option(input: u64) -> Option<CoinOf<T>> {
            input.try_into().ok()
        }

        // from https://stackoverflow.com/questions/56081117
        pub fn coin_to_u64(input: CoinOf<T>) -> Option<u64> {
            TryInto::<u64>::try_into(input).ok()
        }
    }
}
