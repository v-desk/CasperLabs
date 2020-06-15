#![cfg_attr(
    not(target_arch = "wasm32"),
    crate_type = "target arch should be wasm32"
)]
#![no_main]
#![no_std]

extern crate alloc;
extern crate core;

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::convert::TryInto;

use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert
};
use types::{
    ApiError, Key, runtime_args, RuntimeArgs, EntryPoints, 
    EntryPoint, CLType, EntryPointAccess, EntryPointType, CLTyped,
    bytesrepr::{FromBytes, ToBytes}
};

const CONTRACT: &str = "contract";
const CONTRACT_HASH: &str = "contract_hash";
const COUNTER: &str = "counter";
const INCREMENT: &str = "increment";

#[repr(u16)]
enum Error {
    MissingKey = 1,
    UnexpectedType = 2,
    MissingKeyInStorage = 3,
    UnexpectedTypeInStorage = 4
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}

#[no_mangle]
pub extern "C" fn increment() {
    let counter: u64 = key(COUNTER);
    set_key(COUNTER, counter + 1);
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(EntryPoint::new(
        String::from(INCREMENT),
        Vec::new(),
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    let counter = storage::new_uref(10u64);
    let mut keys: BTreeMap<String, Key> = BTreeMap::new();
    keys.insert(String::from(COUNTER), counter.into());

    let contract_hash = storage::new_contract(entry_points, Some(keys), None, None);
    runtime::put_key(CONTRACT, contract_hash.into());
    let contract_hash_pack = storage::new_uref(contract_hash);
    runtime::put_key(CONTRACT_HASH, contract_hash_pack.into());

    runtime::call_contract::<()>(contract_hash, INCREMENT, runtime_args! {});
}

fn key<T: FromBytes + CLTyped>(name: &str) -> T {
    let key = runtime::get_key(name)
        .unwrap_or_revert_with(Error::MissingKey)
        .try_into()
        .unwrap_or_revert_with(Error::UnexpectedType);
    storage::read(key)
        .unwrap_or_revert_with(Error::MissingKeyInStorage)
        .unwrap_or_revert_with(Error::UnexpectedTypeInStorage)
}

fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}
