#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        sp_runtime::traits::{Hash, Zero},
        dispatch::{DispatchResultWithPostInfo, DispatchResult},
        traits::{Currency, ExistenceRequirement, Randomness},
        pallet_prelude::*
    };
    use frame_system::pallet_prelude::*;

    use sp_io::hashing::blake2_128;

    // TODO Part II: Struct for holding Kitty information.

    // TODO Part II: Enum and implementation to handle Gender type in Kitty struct.

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

        // TODO: Part II: Specify the custom types for our runtime.
	}

    // Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
        // TODO Part III
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// TODO Part III
	}

    #[pallet::storage]
    #[pallet::getter(fn count_for_kitties)]
    // Keep track of the number of Kitties in existence.
    pub(super) type CountForKitties<T: Config> = StorageValue<_, u64, ValueQuery>;


    // TODO Part II: Remaining storage items.

    // TODO Part III: Our pallet's genesis configuration.

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
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

pub use pallet::*;
