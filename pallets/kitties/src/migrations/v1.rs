// use frame_support::{
//     pallet_prelude::*,
//     traits::GetStorageVersion,
//     storage::StoragePrefixedMap,
//     weights::Weight,
// };
// use frame_support::{migration::storage_key_iter,Blake2_128Concat};

// #[derive(Encode,Decode,Clone,Copy,Debug,PartialEq,Eq,Default,TypeInfo,MaxEncodedLen)]
// pub struct OldKitty(pub [u8;16]);

// pub fn migrate<T: crate::Config>() -> Weight {
//     let on_chain_version = crate::Pallet::<T>::on_chain_storage_version();
//     let current_version = crate::Pallet::<T>::current_storage_version();

//     if on_chain_version != 0 {
//         return 0;
//     }

//     if current_version != 1 {
//         return 0;
//     }

//     let module = crate::Kitties::<T>::module_prefix();
//     let item = crate::Kitties::<T>::storage_prefix();  
//     for (index,kitty) in storage_key_iter::<crate::KittyId,OldKitty,Blake2_128Concat>(module,item).drain(){
//         let new_kitty = crate::Kitty {
//             dna: kitty.0,
//             name: *b"abcd",
//         };
//         crate::Kitties::<T>::insert(index,new_kitty);
//     }
//     0
// }