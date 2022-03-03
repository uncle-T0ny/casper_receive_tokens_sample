#![no_main]
#![feature(once_cell)]

#![warn(missing_docs)]
#![no_std]

extern crate alloc;

pub mod constants;
mod detail;
pub mod entry_points;
mod error;
mod address;

use alloc::string::{String, ToString};
use core::convert::TryInto;

use once_cell::unsync::OnceCell;

use casper_contract::{contract_api::{runtime, storage}, unwrap_or_revert::UnwrapOrRevert};
use casper_contract::contract_api::system;
use casper_types::{contracts::NamedKeys, EntryPoints, Key, URef, U256, U512, RuntimeArgs, runtime_args, ContractHash, ApiError, CLTyped};

use constants::{
     ERC20_TOKEN_CONTRACT_KEY_NAME
};
pub use error::Error;
use crate::constants::{MAIN_PURSE_KEY_NAME, PACKAGE_HASH_KEY_NAME, RES1_UREF_KEY_NAME, RES_UREF_KEY_NAME};

const RESULT_KEY: &str = "result";
const RESULT_KEY1: &str = "result1";

#[derive(Default)]
pub struct TestingReceive {
    main_purse: OnceCell<URef>,
}

impl TestingReceive {
    fn new(main_purse: URef) -> Self {
        Self {
            main_purse: main_purse.into(),
        }
    }

    pub fn install() -> Result<TestingReceive, Error> {
        let default_entry_points = entry_points::default();
        TestingReceive::install_custom(
            ERC20_TOKEN_CONTRACT_KEY_NAME,
            default_entry_points,
        )
    }

    #[doc(hidden)]
    pub fn install_custom(
        contract_key_name: &str,
        entry_points: EntryPoints,
    ) -> Result<TestingReceive, Error> {

        let mut named_keys = NamedKeys::new();

        let purse = system::create_purse();

        named_keys.insert(MAIN_PURSE_KEY_NAME.to_string(), purse.into());

        let res_uref = storage::new_uref(String::from("res")).into_read_write();
        let res_key = Key::from(res_uref);

        let res1_uref = storage::new_uref(String::from("res1")).into_read_write();
        let res1_key = Key::from(res1_uref);

        named_keys.insert(RESULT_KEY.to_string(), res_key);
        named_keys.insert(RESULT_KEY1.to_string(), res1_key);

        let (package_hash, access_token) = storage::create_contract_package_at_hash();

        named_keys.insert(PACKAGE_HASH_KEY_NAME.to_string(), package_hash.into());

        let purse = system::create_purse();

        named_keys.insert(RES_UREF_KEY_NAME.to_string(), res_uref.into());
        named_keys.insert(RES1_UREF_KEY_NAME.to_string(), res1_uref.into());

        let (contract_hash, _version) =
            storage::new_locked_contract(entry_points, Some(named_keys), None, None);


        // Hash of the installed contract will be reachable through named keys.
        runtime::put_key(contract_key_name, Key::from(contract_hash));

        Ok(TestingReceive::new(
            purse,
        ))
    }
}

// constructor
#[no_mangle]
fn call() {
    let _token = TestingReceive::install().unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn testing_cspr_transfer() {
    let caller_purse: URef = runtime::get_named_arg("purse");
    let amount: U256 = runtime::get_named_arg("amount");

    if !caller_purse.is_readable() {
        runtime::revert(ApiError::from(27));
    }

    if !caller_purse.is_writeable() {
        runtime::revert(ApiError::from(28));
    }

    let key = runtime::get_key(MAIN_PURSE_KEY_NAME);
    let key_uref = key
        .unwrap_or_revert_with(ApiError::from(29));
    let self_purse_uref = key_uref
        .as_uref()
        .unwrap_or_revert_with(ApiError::from(30));

    if !self_purse_uref.is_addable() {
        runtime::revert(ApiError::from(31));
    }

    if !self_purse_uref.is_readable() {
        runtime::revert(ApiError::from(32));
    }

    if !self_purse_uref.is_writeable() {
        runtime::revert(ApiError::from(33));
    }


    let self_purse = self_purse_uref.into_read_add_write();

    // let self_purse: URef = system::create_purse();
    let _: () = system::transfer_from_purse_to_purse(
        caller_purse,
        self_purse,
        U512::from(amount.as_u128()),
        None,
    ).unwrap_or_revert();
}

#[no_mangle]
pub extern "C" fn testing_erc20_transfer() {
    let token: Key = runtime::get_named_arg("token");
    let token_contract_key = ContractHash::from(token.into_hash().unwrap_or_default());
    let package_hash_key = runtime::get_key(PACKAGE_HASH_KEY_NAME).unwrap_or_revert_with(ApiError::from(14));

    let owner = detail::get_immediate_caller_address().unwrap_or_revert_with(ApiError::from(12));
    let owner_hash = owner.as_account_hash()
        .unwrap_or_revert_with(ApiError::from(13));

    let res_uref_key = runtime::get_key(RES_UREF_KEY_NAME).unwrap_or_revert_with(ApiError::from(29));
    let res_uref: URef = res_uref_key.try_into().unwrap_or_revert_with(ApiError::from(30));


    let res1_uref_key = runtime::get_key(RES1_UREF_KEY_NAME).unwrap_or_revert_with(ApiError::from(29));
    let res1_uref: URef = res1_uref_key.try_into().unwrap_or_revert_with(ApiError::from(30));

    if !res_uref.is_writeable() {
        runtime::revert(ApiError::from(35));
    }

    if !res1_uref.is_writeable() {
        runtime::revert(ApiError::from(36));
    }

    storage::write(res_uref, package_hash_key.to_formatted_string());
    storage::write(res1_uref, owner_hash.to_formatted_string());

    let args: RuntimeArgs = runtime_args! {
            "owner" => Key::from(*owner_hash),
            "recipient" => Key::from(Key::from(package_hash_key)),
            "amount" => U256::from(100u128),
        };

    let _: () = runtime::call_contract(
        token_contract_key,
        "transfer_from",
        args,
    );
}
