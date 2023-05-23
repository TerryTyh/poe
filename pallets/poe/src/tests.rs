use crate::{mock::*,Error };
use frame_support::{ assert_ok,BoundedVec, assert_noop};

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0,1]).unwrap();
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
	});
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0,1]).unwrap();
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        assert_noop!(
            PoeModule::create_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ProofAlreadyExist
        );

	});
}

#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0,1]).unwrap();
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));
	});
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0,1]).unwrap();

        assert_noop!(
            PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ClaimNotExist
        );

	});
}

#[test]
fn revoke_claim_failed_with_wrong_owner() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0,1]).unwrap();
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());       

        assert_noop!(
            PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
            Error::<Test>::NotClaimOwner
        );

	});
}

#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0,1]).unwrap();
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(),2));
	});
}

#[test]
fn transfer_claim_failed_when_claim_is_not_exist() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0,1]).unwrap();

        assert_noop!(
            PoeModule::transfer_claim(Origin::signed(1), claim.clone(),2),
            Error::<Test>::ClaimNotExist
        );

	});
}

#[test]
fn transfer_claim_failed_with_wrong_owner() {
	new_test_ext().execute_with(|| {
		let claim = BoundedVec::try_from(vec![0,1]).unwrap();
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());       

        assert_noop!(
            PoeModule::transfer_claim(Origin::signed(2), claim.clone(),1),
            Error::<Test>::NotClaimOwner
        );

	});
}