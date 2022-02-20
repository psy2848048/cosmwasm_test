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

    const TEST_NAME: &str = "Test token";
    const TEST_SYMBOL: &str = "TEST";
    const TEST_DECIMAL: u8 = 9;
    const TEST_ADDRESS: &str = "addr0000";
    const TEST_AMOUNT: u128 = 11223344;

    const TEST_ADDRESS_BALANCE_EMPTY: &str = "addr0001";

    #[test]
    fn works() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: String::from(TEST_NAME),
            symbol: String::from(TEST_SYMBOL),
            decimals: TEST_DECIMAL,
            initial_balances: [InitialBalance {
                address: String::from(TEST_ADDRESS),
                amount: Uint128::from(TEST_AMOUNT),
            }].to_vec(),
        };

        let (env, info) = mock_env_height("creater", 450, 500);
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

        assert_eq!(0, res.messages.len());
        assert_eq!(
            get_constants(&deps.storage),
            Constants {
                decimals: TEST_DECIMAL,
                name: String::from(TEST_NAME),
                symbol: String::from(TEST_SYMBOL),
            },
        );

        assert_eq!(
            get_balance(&deps.storage, &Addr::unchecked(TEST_ADDRESS)),
            TEST_AMOUNT,
        );
    }

    #[test]
    fn works_with_empty_balance() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: String::from(TEST_NAME),
            symbol: String::from(TEST_SYMBOL),
            decimals: TEST_DECIMAL,
            initial_balances: [InitialBalance {
                address: String::from(TEST_ADDRESS),
                amount: Uint128::from(TEST_AMOUNT),
            }].to_vec(),
        };

        let (env, info) = mock_env_height("creater", 450, 500);
        instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

        assert_eq!(
            get_balance(&deps.storage, &Addr::unchecked(TEST_ADDRESS_BALANCE_EMPTY)),
            0,
        );

        assert_eq!(
            get_total_supply(&deps.storage),
            TEST_AMOUNT,
        );
    }
}
