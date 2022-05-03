#![cfg_attr(not(feature = "std"), no_std)]


/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>

#[frame_support::pallet]
pub mod pallet {
    use frame_system::pallet_prelude::*;
    use frame_support::pallet_prelude::*;
    use frame_support::{
        sp_runtime::traits::Hash,
        traits::{Currency, tokens::ExistenceRequirement, Randomness},
        transactional,
    };

    use sp_io::hashing::blake2_128;
    use scale_info::TypeInfo;


    #[cfg(feature = "std")]
    use frame_support::serde::{Deserialize, Serialize};

    type AccountOf<T> = <T as frame_system::Config>::AccountId;
    type BalanceOf<T> = 
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    #[codec(mel_bound())]
    pub struct Kitty<T: Config> {
        pub dna: [u8; 16],
        pub price: Option<BalanceOf<T>>,
        pub gender: Gender,
        pub owner: AccountOf<T>,
    }

    #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub enum Gender {
        Male,
        Female,
    }

    #[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        // The Current handler for the Kitties pallet.
        type Currency: Currency<Self::AccountId>;

        type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;

        #[pallet::constant]
        type MaxKittyOwned: Get<u32>;

        // TODO: Part II: Specify the custom types for our runtime.
	}

    // Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
        CountForKittiesOverflow,
        ExceedMaxKittyOwned,
        BuyerIsKittyOwner,
        TransferToSelf,
        KittyExists,
        KittyNotExist,
        NotKittyOwner,
        KittyNotForSale,
        KittyBidPriceTooLow,
        NotEnoughBalance,
    }

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// TODO Part III
        Created(T::AccountId, T::Hash),
        PriceSet(T::AccountId, T::Hash, Option<BalanceOf<T>>),
        Transferred(T::AccountId, T::AccountId, T::Hash),
        Bought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),
	}

    #[pallet::storage]
    #[pallet::getter(fn count_for_kitties)]
    // Keep track of the number of Kitties in existence.
    pub(super) type CountForKitties<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub(super) type Kitties<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::Hash,
        Kitty<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn kitties_owned)]
    pub(super) type KittiesOwned<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        BoundedVec<T::Hash, T::MaxKittyOwned>,
        ValueQuery,
    >;


    // TODO Part II: Remaining storage items.

    // TODO Part III: Our pallet's genesis configuration.

    // TODO Part II: helper function for Kitty struct

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub kitties: Vec<(T::AccountId, [u8; 16], Gender)>
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> GenesisConfig<T> {
            GenesisConfig {
                kitties: vec![]
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for (acct, dna, gender) in &self.kitties {
                let _ = <Pallet<T>>::mint(acct, Some(dna.clone()), Some(gender.clone()));
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {

    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100)]
        pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::mint(&sender, None, None)?;

            log::info!("A kitty is born with ID: {:?}", kitty_id);
            Self::deposit_event(Event::Created(sender, kitty_id));
            Ok(())
        }

        #[pallet::weight(100)]
        pub fn set_price(
            origin: OriginFor<T>,
            kitty_id: T::Hash,
            new_price: Option<BalanceOf<T>>
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_kitty_owner(&kitty_id, &sender)?, <Error<T>>::NotKittyOwner);

            let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;

            kitty.price = new_price.clone();
            <Kitties<T>>::insert(&kitty_id, kitty);

            Self::deposit_event(Event::PriceSet(sender, kitty_id, new_price));

            Ok(())
        }

        #[pallet::weight(100)]
        pub fn transfer(
            origin: OriginFor<T>,
            to: T::AccountId,
            kitty_id: T::Hash
        ) -> DispatchResult {
            let from = ensure_signed(origin)?;
            
            ensure!(Self::is_kitty_owner(&kitty_id, &from)?, <Error<T>>::NotKittyOwner);
            ensure!(from != to, <Error<T>>::TransferToSelf);

            let to_owned = <KittiesOwned<T>>::get(&to);
            ensure!((to_owned.len() as u32) < T::MaxKittyOwned::get(), <Error<T>>::ExceedMaxKittyOwned);

            Self::transfer_kitty_to(&kitty_id, &to)?;

            Self::deposit_event(Event::Transferred(from, to, kitty_id));

            Ok(())
        }

        #[transactional]
        #[pallet::weight(100)]
        pub fn buy_kitty(
            origin: OriginFor<T>,
            kitty_id: T::Hash,
            bid_price: BalanceOf<T>
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;

            let kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
            ensure!(kitty.owner != buyer, <Error<T>>::BuyerIsKittyOwner);

            if let Some(ask_price) = kitty.price {
                ensure!(ask_price <= bid_price, <Error<T>>::KittyBidPriceTooLow);
            } else {
                Err(<Error<T>>::KittyNotForSale)?;
            }

            ensure!(T::Currency::free_balance(&buyer) >= bid_price, <Error<T>>::NotEnoughBalance);

            let to_owned = <KittiesOwned<T>>::get(&buyer);

            ensure!((to_owned.len() as u32) < T::MaxKittyOwned::get(), <Error<T>>::ExceedMaxKittyOwned);

            let seller = kitty.owner.clone();

            T::Currency::transfer(&buyer, &seller, bid_price, ExistenceRequirement::KeepAlive)?;

            Self::transfer_kitty_to(&kitty_id, &buyer)?;

            Self::deposit_event(Event::Bought(buyer, seller, kitty_id, bid_price));

            Ok(())
        }

        #[pallet::weight(100)]
        pub fn breed_kitty(
            origin: OriginFor<T>,
            parent1: T::Hash,
            parent2: T::Hash
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_kitty_owner(&parent1, &sender)?, <Error<T>>::NotKittyOwner);
            ensure!(Self::is_kitty_owner(&parent2, &sender)?, <Error<T>>::NotKittyOwner);

            let new_dna = Self::breed_dna(&parent1, &parent2)?;
            Self::mint(&sender, Some(new_dna), None)?;

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn gen_gender() -> Gender {
            let random = T::KittyRandomness::random(&b"gender"[..]).0;
            match random.as_ref()[0] % 2 {
                0 => Gender::Male,
                _ => Gender::Female,
            }
        }

        // TODO Part III: helper functions for dispatchable functions

        fn gen_dna() -> [u8; 16] {
            let payload = (
                T::KittyRandomness::random(&b"dna"[..]).0,
                <frame_system::Pallet<T>>::extrinsic_index().unwrap_or_default(),
                <frame_system::Pallet<T>>::block_number(),
            );
            payload.using_encoded(blake2_128)
        }

        pub fn breed_dna(parent1: &T::Hash, parent2: &T::Hash) -> Result<[u8; 16], Error<T>> {
            let dna1 = Self::kitties(parent1).ok_or(<Error<T>>::KittyNotExist)?.dna;
            let dna2 = Self::kitties(parent2).ok_or(<Error<T>>::KittyNotExist)?.dna;

            let mut new_dna = Self::gen_dna();
            for i in 0..new_dna.len() {
                new_dna[i] = (new_dna[i] & dna1[i] | (!new_dna[i] & dna2[i]));
            }
            Ok(new_dna)
        }

        pub fn mint(
            owner: &T::AccountId,
            dna: Option<[u8; 16]>,
            gender: Option<Gender>,
        ) -> Result<T::Hash, Error<T>> {
            let kitty = Kitty::<T> {
                dna: dna.unwrap_or_else(Self::gen_dna),
                price: None,
                gender: gender.unwrap_or_else(Self::gen_gender),
                owner: owner.clone(),
            };

            let kitty_id = T::Hashing::hash_of(&kitty);

            let new_cnt = Self::count_for_kitties().checked_add(1)
                .ok_or(<Error<T>>::CountForKittiesOverflow)?;

            ensure!(Self::kitties(&kitty_id) == None, <Error<T>>::KittyExists);

            <KittiesOwned<T>>::try_mutate(&owner, |kitty_vec| {
                kitty_vec.try_push(kitty_id)
            }).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

            <Kitties<T>>::insert(kitty_id, kitty);
            <CountForKitties<T>>::put(new_cnt);
            Ok(kitty_id)
        }

        pub fn is_kitty_owner(kitty_id: &T::Hash, acct: &T::AccountId) -> Result<bool, Error<T>> {
            match Self::kitties(kitty_id) {
                Some(kitty) => Ok(kitty.owner == *acct),
                None => Err(<Error<T>>::KittyNotExist)
            }
        }

        #[transactional]
        pub fn transfer_kitty_to(
            kitty_id: &T::Hash,
            to: &T::AccountId,
        ) -> Result<(), Error<T>> {
            let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
            let prev_owner = kitty.owner.clone();
            <KittiesOwned<T>>::try_mutate(&prev_owner, |owned| {
                if let Some(ind) = owned.iter().position(|&id| id == *kitty_id) {
                    owned.swap_remove(ind);
                    return Ok(());
                }
                Err(())
            }).map_err(|_| <Error<T>>::KittyNotExist)?;
            kitty.owner = to.clone();
            kitty.price = None;

            <Kitties<T>>::insert(kitty_id, kitty);

            <KittiesOwned<T>>::try_mutate(to, |vec| {
                vec.try_push(*kitty_id)
            }).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

            Ok(())
        }
    }
}

pub use pallet::*;
