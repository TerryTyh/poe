use crate::*;
use frame_benchmarking::{benchmarks, whitelisted_caller,account};
use frame_system::RawOrigin;

benchmarks! {
     create_claim {
        let d in 0 .. T::MaxClaimLength::get();
        let claim: BoundedVec<u8,T::MaxClaimLength> = BoundedVec::try_from(vec![0;d as usize]).unwrap();
        let caller: T::AccountId = whitelisted_caller();
     }: _(RawOrigin::Signed(caller),claim)

     revoke_claim {
        let d in 0 .. T::MaxClaimLength::get();
        let claim: BoundedVec<u8,T::MaxClaimLength> = BoundedVec::try_from(vec![0;d as usize]).unwrap();
        let caller: T::AccountId = whitelisted_caller();
        assert!(Pallet::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone()).is_ok());
     }: _(RawOrigin::Signed(caller),claim)

     transfer_claim {
      let d in 0 .. T::MaxClaimLength::get();
      let claim: BoundedVec<u8,T::MaxClaimLength> = BoundedVec::try_from(vec![0;d as usize]).unwrap();
      let caller: T::AccountId = whitelisted_caller();
      assert!(Pallet::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone()).is_ok());
      let dest = account("target",0,0);
   }: _(RawOrigin::Signed(caller),claim,dest)

     impl_benchmark_test_suite!(PoeModule,crate::mock::new_test_ext(),crate::mock::Test);
}