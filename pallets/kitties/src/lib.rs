#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
  use frame_support::pallet_prelude::{*, OptionQuery, DispatchResult};
  use frame_system::pallet_prelude::*;

  use sp_io::hashing::blake2_128;
  use frame_support::traits::Randomness;

  // 参数定义
  pub type KittyId = u32;
  #[derive(Encode,Decode,Clone,Copy,RuntimeDebug,PartialEq,Eq,Default,TypeInfo,MaxEncodedLen)]
  pub struct Kitty(pub [u8;16]);

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::config]  // <-- Step 2. code block will replace this.
  pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
  }

  // 链上存储（及方法定义）.
  #[pallet::storage]
  #[pallet::getter(fn next_kitty_id)]
  pub type NextKittyId<T> = StorageValue<_,KittyId,ValueQuery>;
  
  #[pallet::storage]
  #[pallet::getter(fn kitties)]
  pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyId, Kitty>;

  #[pallet::storage]
  #[pallet::getter(fn kitty_owner)]
  pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyId, T::AccountId>;

  #[pallet::storage]
  #[pallet::getter(fn kitty_parents)]
  pub type KittyParents<T: Config> = StorageMap<_, Blake2_128Concat,KittyId,(KittyId,KittyId),OptionQuery>;

  // 事件
  #[pallet::event]   // <-- Step 3. code block will replace this.
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
	/// Event emitted when a claim has been created.
	KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty:Kitty },
	KittyBred {who: T::AccountId,kitty_id: KittyId, kitty:Kitty},
	KittyTransferred{who: T::AccountId, recipient: T::AccountId, kitty_id: KittyId}
  }
  // 错误
  #[pallet::error]   // <-- Step 4. code block will replace this.
  pub enum Error<T> {
	InvalidKittyId,
	SameKittyId,
	NotOwner,
  }

  // 请求
  #[pallet::call]    // <-- Step 6. code block will replace this.
  impl<T: Config> Pallet<T> {

	#[pallet::call_index(0)]
	#[pallet::weight(10_000)]
	pub fn create(origin: OriginFor<T>) -> DispatchResult {
	  // Check that the extrinsic was signed and get the signer.
	  // This function will return an error if the extrinsic is not signed.
	  let who = ensure_signed(origin)?;
	  let kitty_id = Self::get_next_id()?;
	  let kitty = Kitty(Self::random_value(&who));

	  Kitties::<T>::insert(kitty_id,&kitty);
	  KittyOwner::<T>::insert(kitty_id,&who);
  
	  Self::deposit_event(Event::KittyCreated { who, kitty_id,kitty });
  
	  Ok(())
	}

	#[pallet::call_index(1)]
	#[pallet::weight(10_000)]
	pub fn breed(origin: OriginFor<T>, kitty_id_1:KittyId, kitty_id_2:KittyId ) -> DispatchResult {
		let who = ensure_signed(origin)?;
		ensure!(kitty_id_1!=kitty_id_2,Error::<T>::SameKittyId);
		ensure!(Kitties::<T>::contains_key(kitty_id_1),Error::<T>::InvalidKittyId);
		ensure!(Kitties::<T>::contains_key(kitty_id_2),Error::<T>::InvalidKittyId);

		let kitty_id = Self::get_next_id()?;
		let kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
		let kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

		let selector = Self::random_value(&who);
		let mut data = [0u8; 16];
		for i in 0..kitty_1.0.len() {
			data[i] = (kitty_1.0[i] & selector[i]) | (kitty_2.0[i] & !selector[i]);
		}
		let kitty = Kitty(data);

		Kitties::<T>::insert(kitty_id,&kitty);
		KittyOwner::<T>::insert(kitty_id,&who);
		KittyParents::<T>::insert(kitty_id,(kitty_id_1,kitty_id_2));
		Self::deposit_event(Event::KittyBred{who,kitty_id,kitty});
		Ok(())
	}

	#[pallet::call_index(2)]
	#[pallet::weight(10_000)]
	pub fn transfer(origin: OriginFor<T>, recipient: T::AccountId, kitty_id:KittyId ) -> DispatchResult {
		let who = ensure_signed(origin)?;
		ensure!(KittyOwner::<T>::contains_key(kitty_id),Error::<T>::InvalidKittyId);

		let owner = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
		ensure!(owner == who, Error::<T>::NotOwner);
		
		KittyOwner::<T>::insert(kitty_id, &recipient);
		Self::deposit_event(Event::KittyTransferred{who,recipient,kitty_id});
		Ok(())
	}

  }

  // 方法实现
  impl <T:Config> Pallet<T> {
	fn get_next_id() -> Result<KittyId, DispatchError> {
		NextKittyId::<T>::try_mutate(|next_id| -> Result<KittyId,DispatchError>{
			let current_id = *next_id;
			*next_id = next_id.checked_add(1).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
			Ok(current_id)
		})
	}

	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
			T::Randomness::random_seed(),
			&sender,
			<frame_system::Pallet<T>>::extrinsic_index(),
		);
		payload.using_encoded(blake2_128)
	}
  }
}