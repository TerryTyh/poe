use crate::{Error, mock::*};
use frame_support::{assert_ok,assert_noop};

#[test]
fn it_works_for_create(){
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        let name = [1,2,1,2,0,0,0,0];
        assert_eq!(KittiesModule::next_kitty_id(),kitty_id);

        let _res = Balances::set_balance(
            Origin::root(),
            account_id,
            1_000_000_000,
            0);

        assert_ok!(KittiesModule::create(Origin::signed(account_id),name));

        assert_eq!(KittiesModule::next_kitty_id(),kitty_id+1);  
        assert_eq!(KittiesModule::kitties(kitty_id).is_some(),true);
        assert_eq!(KittiesModule::kitty_owner(kitty_id),Some(account_id));
        assert_eq!(KittiesModule::kitty_parents(kitty_id),None);

        crate::NextKittyId::<Test>::set(crate::KittyId::max_value());
        assert_noop!(
            KittiesModule::create(Origin::signed(account_id),name),
            Error::<Test>::InvalidKittyId
        );
        // 判断是否启用了KittyCreated事件
        System::assert_has_event(
            crate::Event::KittyCreated { 
                who: account_id, 
                kitty_id: kitty_id, 
                kitty:KittiesModule::kitties(kitty_id).unwrap()
            }.into()
        );

        // 判断最后一个事件是否是KittyCreated事件
        System::assert_last_event(
            crate::Event::KittyCreated { 
                who: account_id, 
                kitty_id: kitty_id, 
                kitty:KittiesModule::kitties(kitty_id).unwrap()
            }.into()
        );
    }); 
        


}

#[test]
fn it_works_for_breed(){
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        let name = [1,2,3,4,0,0,0,0];
        let _res = Balances::set_balance(
            Origin::root(),
            account_id,
            1_000_000_000,
            0);

        assert_noop!(
            KittiesModule::breed(Origin::signed(account_id),kitty_id,kitty_id,name),
            Error::<Test>::SameKittyId
        );

        assert_noop!(
            KittiesModule::breed(Origin::signed(account_id),kitty_id,kitty_id+1,name),
            Error::<Test>::InvalidKittyId
        );

        assert_ok!(KittiesModule::create(Origin::signed(account_id),name));
        assert_ok!(KittiesModule::create(Origin::signed(account_id),name));

        assert_eq!(KittiesModule::next_kitty_id(),kitty_id + 2);

        assert_ok!(KittiesModule::breed(
            Origin::signed(account_id), 
            kitty_id, 
            kitty_id+1,
            name
        ));

        let breed_kitty_id = 2;
        assert_eq!(KittiesModule::next_kitty_id(),breed_kitty_id+1);
        assert_eq!(KittiesModule::kitties(breed_kitty_id).is_some(),true);
        assert_eq!(KittiesModule::kitty_owner(breed_kitty_id),Some(account_id)); 
        assert_eq!(
            KittiesModule::kitty_parents(breed_kitty_id),
            Some((kitty_id,kitty_id +1))
        );

        // 判断是否启用了KittyBred事件
        System::assert_has_event(
            crate::Event::KittyBred { 
                who: account_id, 
                kitty_id: breed_kitty_id, 
                kitty:KittiesModule::kitties(kitty_id).unwrap()
            }.into()
        );
        // 判断最后一个事件是否是KittyBred事件
        System::assert_last_event(
            crate::Event::KittyBred { 
                who: account_id, 
                kitty_id: breed_kitty_id, 
                kitty:KittiesModule::kitties(kitty_id).unwrap()
            }.into()
        );
    });
}

#[test]
fn it_works_for_transfer(){
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        let recipient = 2;
        let name = [1,2,3,4,0,0,0,0];
        let _res = Balances::set_balance(
            Origin::root(),
            account_id,
            1_000_000_000,
            0);
        assert_ok!(KittiesModule::create(Origin::signed(account_id),name));
        assert_eq!(KittiesModule::kitty_owner(kitty_id),Some(account_id));

        assert_noop!(KittiesModule::transfer(
            Origin::signed(recipient), 
            recipient, 
            kitty_id
        ), Error::<Test>::NotOwner);  

        assert_ok!(KittiesModule::transfer(
            Origin::signed(account_id), 
            recipient, 
            kitty_id
        ));      

        assert_eq!(KittiesModule::kitty_owner(kitty_id),Some(recipient));

        assert_ok!(KittiesModule::transfer(
            Origin::signed(recipient), 
            account_id, 
            kitty_id
        )); 

        assert_eq!(KittiesModule::kitty_owner(kitty_id),Some(account_id));
    
        // 判断是否启用了KittyTransferred事件,完成从account_id向recipient 转移kitty
        System::assert_has_event(
            crate::Event::KittyTransferred { 
                who: account_id, 
                recipient: recipient,
                kitty_id: kitty_id
            }.into()
        );

        // 判断是否启用了KittyTransferred事件,完成从recipient向account_id 返还kitty
        System::assert_has_event(
            crate::Event::KittyTransferred { 
                who: recipient, 
                recipient: account_id,
                kitty_id: kitty_id
            }.into()
        );

        // 判断最后一个事件是否是KittyTransferred事件：从recipient向account_id 返还kitty
        System::assert_last_event(
            crate::Event::KittyTransferred { 
                who: recipient, 
                recipient: account_id,
                kitty_id: kitty_id
            }.into()
        );
    });
}


#[test]
fn it_works_for_sale(){
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let account_id = 1;
        let name = [1,2,3,4,0,0,0,0];
        let _res = Balances::set_balance(
            Origin::root(),
            account_id,
            1_000_000_000,
            0);
        assert_ok!(KittiesModule::create(Origin::signed(account_id),name));
        assert_eq!(KittiesModule::kitty_owner(kitty_id),Some(account_id));
        assert_eq!(KittiesModule::kitty_on_sale(kitty_id).is_some(),false);
        assert_ok!(KittiesModule::sale(Origin::signed(account_id), kitty_id));
        assert_eq!(KittiesModule::kitty_on_sale(kitty_id).is_some(),true);

        System::assert_has_event(
            crate::Event::KittyOnSale{
                who: account_id,
                kitty_id:kitty_id
            }.into()
        );

        System::assert_last_event(
            crate::Event::KittyOnSale{
                who: account_id,
                kitty_id:kitty_id
            }.into()
        );
    });
}

#[test]
fn it_works_for_buy(){
    new_test_ext().execute_with(|| {
        let kitty_id = 0;
        let saler_id = 0;
        let buyer_id = 1;
        let name = [1,2,3,4,0,0,0,0];
        let _res = Balances::set_balance(
            Origin::root(),
            saler_id,
            1_000_000_000,
            0);

        let _res = Balances::set_balance(
            Origin::root(),
            buyer_id,
            1_000_000_000,
            0);

        assert_ok!(KittiesModule::create(Origin::signed(saler_id),name));
        assert_eq!(KittiesModule::kitty_owner(kitty_id),Some(saler_id));
        assert_ok!(KittiesModule::sale(Origin::signed(saler_id), kitty_id));
        assert_eq!(KittiesModule::kitty_on_sale(kitty_id).is_some(),true);

        assert_ok!(KittiesModule::buy(Origin::signed(buyer_id), kitty_id));
        assert_eq!(KittiesModule::kitty_on_sale(kitty_id).is_some(),false);

        System::assert_has_event(
            crate::Event::KittyBought{
                who: buyer_id,
                kitty_id:kitty_id
            }.into()
        );

        System::assert_last_event(
            crate::Event::KittyBought{
                who: buyer_id,
                kitty_id:kitty_id
            }.into()
        );
    });
}