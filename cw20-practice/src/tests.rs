use crate::contract::*;
use crate::state::*;
use crate::msg::*;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_slice, Addr, Env, MessageInfo, Storage, Timestamp, Uint128};
use cosmwasm_storage::ReadonlyPrefixedStorage;

fn mock_env_height(signer: &str, height: u64, time: u64) -> (Env, MessageInfo) {
    let mut env = mock_env();
    let info = mock_info(signer, &[]);
    env.block.height = height;
    env.block.time = Timestamp::from_seconds(time);
    (env, info)
}

fn get_constants(storage: &dyn Storage) -> Constants {
    let config_storage = ReadonlyPrefixedStorage::new(storage, PREFIX_CONFIG);
    let data = config_storage
        .get(KEY_CONSTANTS)
        .expect("no config data stored");
    from_slice(&data).expect("invalid data")
}

fn get_total_supply(storage: &dyn Storage) -> u128 {
    let config_storage = ReadonlyPrefixedStorage::new(storage, PREFIX_CONFIG);
    let data = config_storage
        .get(KEY_TOTAL_SUPPLY)
        .expect("no decimals data stored");
    return bytes_to_u128(&data).unwrap();
}

fn get_balance(storage: &dyn Storage, address: &Addr) -> u128 {
    let balances_storage = ReadonlyPrefixedStorage::new(storage, PREFIX_BALANCES);
    return read_u128(&balances_storage, address).unwrap();
}

fn get_allowance(storage: &dyn Storage, owner: &Addr, spender: &Addr) -> u128 {
    let owner_storage = ReadonlyPrefixedStorage::multilevel(
        storage,
        &[PREFIX_ALLOWANCES, owner.as_str().as_bytes()],
    );
    return read_u128(&owner_storage, spender).unwrap();
}

mod instantiate {
    use super::*;
    use crate::error::ContractError;

    #[test]
    fn works() {
        let test_name = "Test token".to_string();
        let test_symbol = "TEST".to_string();
        let test_decimals = 9u8;
        let test_address = "addr0000".to_string();
        let test_amount = 11223344u128;

        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: test_name.clone(),
            symbol: test_symbol.clone(),
            decimals: test_decimals.clone(),
            initial_balances: [InitialBalance {
                address: test_address.clone(),
                amount: Uint128::from(test_amount),
            }].to_vec(),
        };

        let (env, info) = mock_env_height("creater", 450, 500);
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

        assert_eq!(0, res.messages.len());
        assert_eq!(
            get_constants(&deps.storage),
            Constants {
                decimals: test_decimals,
                name: test_name,
                symbol: test_symbol,
            },
        );

        assert_eq!(
            get_balance(&deps.storage, &Addr::unchecked(test_address)),
            test_amount,
        );
    }
}
