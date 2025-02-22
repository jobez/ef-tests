use ef_tests::models::Account;
use kakarot_test_utils::hive_utils::madara::utils::{
    genesis_set_bytecode, genesis_set_storage_kakarot_contract_account,
};
use reth_primitives::Bytes;
use starknet::core::types::FieldElement;
use starknet_api::{hash::StarkFelt, state::StorageKey as StarknetStorageKey};

use crate::models::error::RunnerError;

use super::{
    generate_evm_address_storage, generate_is_initialized_storage, madara_to_katana_storage,
    starknet_storage_key_value,
};

pub(crate) fn generate_evm_contract_storage(
    account: &Account,
) -> Result<Vec<(StarknetStorageKey, StarkFelt)>, RunnerError> {
    account
        .storage
        .iter()
        .flat_map(|(key, value)| {
            // Call genesis_set_storage_kakarot_contract_account util to get the storage tuples
            let storage =
                genesis_set_storage_kakarot_contract_account(FieldElement::ZERO, key.0, value.0);
            match madara_to_katana_storage(storage) {
                Ok(storage) => storage.into_iter().map(Ok).collect::<Vec<_>>(),
                Err(err) => vec![Err(err)],
            }
        })
        .collect::<Result<Vec<(StarknetStorageKey, StarkFelt)>, RunnerError>>()
}

/// Initializes the contract account.
/// Writes the bytecode and the owner to a hashmap.
pub fn initialize_contract_account(
    kakarot_address: FieldElement,
    evm_address: FieldElement,
    bytecode: &Bytes,
) -> Result<Vec<(StarknetStorageKey, StarkFelt)>, RunnerError> {
    let mut contract_storage = vec![
        generate_evm_address_storage(evm_address)?,
        generate_is_initialized_storage()?,
        owner_storage(kakarot_address)?,
    ];
    contract_storage.append(&mut bytecode_storage(bytecode)?);
    Ok(contract_storage)
}

/// Returns the bytecode storage tuples.
fn bytecode_storage(bytecode: &Bytes) -> Result<Vec<(StarknetStorageKey, StarkFelt)>, RunnerError> {
    let bytecode_len = bytecode.len();

    let bytecode = genesis_set_bytecode(bytecode, FieldElement::ZERO);
    let mut bytecode_storage = madara_to_katana_storage(bytecode)?;

    bytecode_storage.push(starknet_storage_key_value(
        "bytecode_len_",
        &[],
        FieldElement::from(bytecode_len),
    )?);

    Ok(bytecode_storage)
}

/// Returns the owner storage tuple.
fn owner_storage(
    kakarot_address: FieldElement,
) -> Result<(StarknetStorageKey, StarkFelt), RunnerError> {
    starknet_storage_key_value("Ownable_owner", &[], kakarot_address)
}
