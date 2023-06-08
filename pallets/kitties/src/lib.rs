#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

mod migrations;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
  use frame_support::{pallet_prelude::{*, OptionQuery, DispatchResult}, traits::{StorageVersion, Hooks}};
  use frame_system::pallet_prelude::{*, BlockNumberFor};
  use frame_support::PalletId;

  use sp_io::hashing::blake2_128;
  use frame_support::traits::{Randomness,Currency,ExistenceRequirement};
  use sp_runtime::traits::AccountIdConversion;
  use crate::migrations;
  // 参数定义
  pub type KittyId = u32;
  pub type BalanceOf<T> = <<T as Config>::Currency as Currency <<T as frame_system::Config>::AccountId>>::Balance;

  #[derive(Encode,Decode,Clone,Copy,RuntimeDebug,PartialEq,Eq,Default,TypeInfo,MaxEncodedLen)]
  //pub struct Kitty(pub [u8;16]);
  pub struct Kitty{
	pub dna: [u8;16],
	pub name: [u8;8],
  }
  
  const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

  #[pallet::pallet]
  #[pallet::storage_version(STORAGE_VERSION)]
  pub struct Pallet<T>(_);

  #[pallet::config]  // <-- Step 2. code block will replace this.
  pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	type Currency: Currency<Self::AccountId>;
	#[pallet::constant]
	type KittyPrice: Get<BalanceOf<Self>>;
	type PalletId: Get<PalletId>;
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

  #[pallet::storage]
  #[pallet::getter(fn kitty_on_sale)]
  pub type KittyOnSale<T: Config> = StorageMap<_, Blake2_128Concat,KittyId,()>;

  // 事件
  #[pallet::event]   // <-- Step 3. code block will replace this.
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
	/// Event emitted when a claim has been created.
	KittyCreated { who: T::AccountId, kitty_id: KittyId, kitty:Kitty },
	KittyBred {who: T::AccountId,kitty_id: KittyId, kitty:Kitty},
	KittyTransferred{who: T::AccountId, recipient: T::AccountId, kitty_id: KittyId},
	KittyOnSale {who: T::AccountId, kitty_id: KittyId},
	KittyBought {who: T::AccountId, kitty_id: KittyId},
  }
  // 错误
  #[pallet::error]   // <-- Step 4. code block will replace this.
  pub enum Error<T> {
	InvalidKittyId,
	SameKittyId,
	NotOwner,
	AlreadyOnSale,
	NotOnSale,
	NoOwner,
	AlreadyOwned,
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
	fn on_runtime_upgrade() -> Weight {
		migrations::v2::migrate::<T>()
	}
  }

  // 请求
  #[pallet::call]    // <-- Step 6. code block will replace this.
  impl<T: Config> Pallet<T> {

	#[pallet::call_index(0)]
	#[pallet::weight(10_000)]
	pub fn create(origin: OriginFor<T>, name: [u8; 8]) -> DispatchResult {
	  // Check that the extrinsic was signed and get the signer.
	  // This function will return an error if the extrinsic is not signed.
	  let who = ensure_signed(origin)?;
	  let kitty_id = Self::get_next_id()?;
	  let dna = Self::random_value(&who);
	  let kitty = Kitty{dna,name};

	  let price = T::KittyPrice::get();
	  //T::Currency::reserve(&who,price)?;
	  T::Currency::transfer(&who,&Self::get_account_id(),price,ExistenceRequirement::KeepAlive)?;
	  Kitties::<T>::insert(kitty_id,&kitty);
	  KittyOwner::<T>::insert(kitty_id,&who);
  
	  Self::deposit_event(Event::KittyCreated { who, kitty_id,kitty });
  
	  Ok(())
	}

	#[pallet::call_index(1)]
	#[pallet::weight(10_000)]
	pub fn breed(origin: OriginFor<T>, kitty_id_1:KittyId, kitty_id_2:KittyId,name: [u8;8] ) -> DispatchResult {
		let who = ensure_signed(origin)?;
		ensure!(kitty_id_1!=kitty_id_2,Error::<T>::SameKittyId);
		ensure!(Kitties::<T>::contains_key(kitty_id_1),Error::<T>::InvalidKittyId);
		ensure!(Kitties::<T>::contains_key(kitty_id_2),Error::<T>::InvalidKittyId);

		let kitty_id = Self::get_next_id()?;
		let kitty_1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
		let kitty_2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

		let selector = Self::random_value(&who);
		let mut dna = [0u8; 16];
		for i in 0..kitty_1.dna.len() {
			dna[i] = (kitty_1.dna[i] & selector[i]) | (kitty_2.dna[i] & !selector[i]);
		 }
		let kitty = Kitty{dna,name};

		let price = T::KittyPrice::get();
		//T::Currency::reserve(&who,price)?;
		T::Currency::transfer(&who,&Self::get_account_id(),price,ExistenceRequirement::KeepAlive)?;

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

	#[pallet::call_index(3)]
	#[pallet::weight(10_000)]
	pub fn sale(origin: OriginFor<T>,kitty_id:KittyId,) -> DispatchResult {
		let who = ensure_signed(origin)?;
		Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;

		ensure!(Self::kitty_owner(kitty_id) == Some(who.clone()),Error::<T>::NotOwner);
		ensure!(Self::kitty_on_sale(kitty_id).is_none(),Error::<T>::AlreadyOnSale);

		<KittyOnSale::<T>>::insert(kitty_id,());
		Self::deposit_event(Event::KittyOnSale{who,kitty_id});
		Ok(())
	}

	#[pallet::call_index(4)]
	#[pallet::weight(10_000)]
	pub fn buy(origin: OriginFor<T>,kitty_id:KittyId,) -> DispatchResult {
		let who = ensure_signed(origin)?;
		Self::kitties(kitty_id).ok_or::<DispatchError>(Error::<T>::InvalidKittyId.into())?;
		let owner = Self::kitty_owner(kitty_id).ok_or::<DispatchError>(Error::<T>::NoOwner.into())?;
		ensure!(owner != who, Error::<T>::AlreadyOwned);
		ensure!(Self::kitty_on_sale(kitty_id).is_some(),Error::<T>::NotOnSale);

		let price = T::KittyPrice::get();
		// T::Currency::reserve(&who,price)?;
		// T::Currency::unreserve(&owner,price);
		T::Currency::transfer(&who,&owner,price,ExistenceRequirement::KeepAlive)?;
		<KittyOwner::<T>>::insert(kitty_id,&who);
		<KittyOnSale::<T>>::remove(kitty_id);
		Self::deposit_event(Event::KittyBought{who,kitty_id});
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

	pub fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
			T::Randomness::random_seed(),
			&sender,
			<frame_system::Pallet<T>>::extrinsic_index(),
		);
		payload.using_encoded(blake2_128)
	}

	pub fn get_account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}
  }
}