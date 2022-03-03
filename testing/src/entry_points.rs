//! Contains definition of the entry points.
use alloc::{string::String, vec};

use casper_types::{CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter, U256, URef, Key};

use crate::constants::{TESTING_CSPR_TRANSFER_ENTRY_POINT_NAME, TESTING_ERC20_TRANSFER_ENTRY_POINT_NAME};

/// test
pub fn testing_cspr_transfer() -> EntryPoint {
    EntryPoint::new(
        String::from(TESTING_CSPR_TRANSFER_ENTRY_POINT_NAME),
        vec![
            Parameter::new("purse", URef::cl_type()),
            Parameter::new("amount", U256::cl_type()),
        ],
        u8::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// test
pub fn testing_erc20_transfer() -> EntryPoint {
    EntryPoint::new(
        String::from(TESTING_ERC20_TRANSFER_ENTRY_POINT_NAME),
        vec![
            Parameter::new("token", Key::cl_type()),
            Parameter::new("value", U256::cl_type()),
        ],
        u8::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the default set of ERC20 token entry points.
pub fn default() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(testing_cspr_transfer());
    entry_points.add_entry_point(testing_erc20_transfer());

    entry_points
}
