use frame_support::{
    pallet_prelude::*,
    traits::GetStorageVersion,
    storage::StoragePrefixedMap,
    weights::Weight,
};
use frame_support::{migration::storage_key_iter,Blake2_128Concat};

#[derive(Encode,Decode,Clone,Copy,Debug,PartialEq,Eq,Default,TypeInfo,MaxEncodedLen)]
pub struct OldKittyV0(pub [u8;16]);


#[derive(Encode,Decode,Clone,Copy,Debug,PartialEq,Eq,Default,TypeInfo,MaxEncodedLen)]
pub struct OldKittyV1{
	pub dna: [u8;16],
	pub name: [u8;4],
  }

pub fn migrate<T: crate::Config>() -> Weight {
    let on_chain_version = crate::Pallet::<T>::on_chain_storage_version();
    let current_version = crate::Pallet::<T>::current_storage_version();

    //如果链上版本大于1，则无需在此升级
    if on_chain_version > 1 {
        return 0;
    }
    // 如果当前版本不等于2，则无需在此升级
    if current_version != 2 {
        return 0;
    }

    if on_chain_version == 0   {
        v0_to_2::<T>();
    }
    if on_chain_version == 1 {
        v1_to_2::<T>();
    }
    0

    
}

fn v0_to_2<T: crate::Config>() {
    let module = crate::Kitties::<T>::module_prefix();
    let item = crate::Kitties::<T>::storage_prefix();  
    for (index,kitty) in storage_key_iter::<crate::KittyId,OldKittyV0,Blake2_128Concat>(module,item).drain(){
        let new_kitty_1 = crate::Kitty {
            dna: kitty.0,
            name: *b"abcd0000",
        };
        crate::Kitties::<T>::insert(index,new_kitty_1);
    }
}

fn v1_to_2<T: crate::Config>() {
    let module = crate::Kitties::<T>::module_prefix();
    let item = crate::Kitties::<T>::storage_prefix();  
    for (index,kitty) in storage_key_iter::<crate::KittyId,OldKittyV1,Blake2_128Concat>(module,item).drain(){
        let temp_name = [kitty.name,*b"0000"].concat();
        let mut new_name:[u8;8] = [0;8];
        for (i,_) in temp_name.iter().enumerate() {
            new_name[i] = temp_name[i]
        }

        let new_kitty_2 = crate::Kitty {
            dna: kitty.dna,
            name:new_name,
        };

        crate::Kitties::<T>::insert(index,new_kitty_2);
    }
}