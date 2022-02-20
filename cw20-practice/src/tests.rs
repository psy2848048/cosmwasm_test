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
    const TEST_AMOUNT: u128 = 10;

    const TEST_ADDRESS_BALANCE_EMPTY: &str = "addr0001";

    const TEST_ADDRESS_2: &str = "addr002";
    const TEST_AMOUNT_2: u128 = 20;

    const TEST_ADDRESS_3: &str = "addr003";
    const TEST_AMOUNT_3: u128 = 30;

    const TEST_ADDRESS_4: &str = "addr004";
    const TEST_AMOUNT_4: u128 = 40;

    const TEST_TOTAL_SUPPLY: u128 = TEST_AMOUNT + TEST_AMOUNT_2 + TEST_AMOUNT_3 + TEST_AMOUNT_4;

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

    #[test]
    fn works_with_multiple_balance() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: String::from(TEST_NAME),
            symbol: String::from(TEST_SYMBOL),
            decimals: TEST_DECIMAL,
            initial_balances: [
                InitialBalance {
                    address: String::from(TEST_ADDRESS),
                    amount: Uint128::from(TEST_AMOUNT),
                },
                InitialBalance {
                    address: String::from(TEST_ADDRESS_2),
                    amount: Uint128::from(TEST_AMOUNT_2),
                },
                InitialBalance {
                    address: String::from(TEST_ADDRESS_3),
                    amount: Uint128::from(TEST_AMOUNT_3),
                },
                InitialBalance {
                    address: String::from(TEST_ADDRESS_4),
                    amount: Uint128::from(TEST_AMOUNT_4),
                },
            ].to_vec(),
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
            get_total_supply(&deps.storage),
            TEST_TOTAL_SUPPLY,
        );

        assert_eq!(get_balance(&deps.storage, &Addr::unchecked(TEST_ADDRESS)), TEST_AMOUNT);
        assert_eq!(get_balance(&deps.storage, &Addr::unchecked(TEST_ADDRESS_2)), TEST_AMOUNT_2);
        assert_eq!(get_balance(&deps.storage, &Addr::unchecked(TEST_ADDRESS_3)), TEST_AMOUNT_3);
        assert_eq!(get_balance(&deps.storage, &Addr::unchecked(TEST_ADDRESS_4)), TEST_AMOUNT_4);
    }

    #[test]
    fn works_with_balance_larger_than_53_bit() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: String::from(TEST_NAME),
            symbol: String::from(TEST_SYMBOL),
            decimals: TEST_DECIMAL,
            initial_balances: [InitialBalance {
                address: String::from(TEST_ADDRESS),
                amount: Uint128::from(9007199254740993u128), // this value cannot be represented in JS (float)
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
            9007199254740993,
        );
        assert_eq!(get_total_supply(&deps.storage), 9007199254740993);
    }

    #[test]
    fn works_with_balance_larger_than_64_bit() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: String::from(TEST_NAME),
            symbol: String::from(TEST_SYMBOL),
            decimals: TEST_DECIMAL,
            initial_balances: [InitialBalance {
                address: String::from(TEST_ADDRESS),
                amount: Uint128::from(100000000000000000000000000u128), // this value cannot be represented in JS (float)
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
            100000000000000000000000000u128,
        );
        assert_eq!(get_total_supply(&deps.storage), 100000000000000000000000000u128);
    }

    #[test]
    fn fails_for_large_decimals() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: String::from(TEST_NAME),
            symbol: String::from(TEST_SYMBOL),
            decimals: 55,
            initial_balances: [
                InitialBalance {
                    address: String::from(TEST_ADDRESS),
                    amount: Uint128::from(TEST_AMOUNT),
                },
            ].to_vec(),
        };

        let (env, info) = mock_env_height("creator", 450, 550);
        let result = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match result {
            Ok(_) => panic!("expected error"),
            Err(ContractError::DecimalsExceeded {}) => {}
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn fails_for_too_short_name() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: String::from("a"),
            symbol: String::from(TEST_SYMBOL),
            decimals: 55,
            initial_balances: [
                InitialBalance {
                    address: String::from(TEST_ADDRESS),
                    amount: Uint128::from(TEST_AMOUNT),
                },
            ].to_vec(),
        };

        let (env, info) = mock_env_height("creator", 450, 550);
        let result = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match result {
            Ok(_) => panic!("expected error"),
            Err(ContractError::NameWrongFormat {}) => {}
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn fails_for_too_long_name() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: String::from("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            symbol: String::from(TEST_SYMBOL),
            decimals: 55,
            initial_balances: [
                InitialBalance {
                    address: String::from(TEST_ADDRESS),
                    amount: Uint128::from(TEST_AMOUNT),
                },
            ].to_vec(),
        };

        let (env, info) = mock_env_height("creator", 450, 550);
        let result = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match result {
            Ok(_) => panic!("expected error"),
            Err(ContractError::NameWrongFormat {}) => {}
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn fails_for_too_short_symbol() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: String::from(TEST_NAME),
            symbol: String::from("A"),
            decimals: 55,
            initial_balances: [
                InitialBalance {
                    address: String::from(TEST_ADDRESS),
                    amount: Uint128::from(TEST_AMOUNT),
                },
            ].to_vec(),
        };

        let (env, info) = mock_env_height("creator", 450, 550);
        let result = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match result {
            Ok(_) => panic!("expected error"),
            Err(ContractError::TickerWrongSymbolFormat {}) => {}
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn fails_for_too_long_symbol() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: String::from(TEST_NAME),
            symbol: String::from("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
            decimals: 55,
            initial_balances: [
                InitialBalance {
                    address: String::from(TEST_ADDRESS),
                    amount: Uint128::from(TEST_AMOUNT),
                },
            ].to_vec(),
        };

        let (env, info) = mock_env_height("creator", 450, 550);
        let result = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match result {
            Ok(_) => panic!("expected error"),
            Err(ContractError::TickerWrongSymbolFormat {}) => {}
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    #[test]
    fn fails_for_too_lower_case() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            name: String::from(TEST_NAME),
            symbol: String::from("aaa"),
            decimals: 55,
            initial_balances: [
                InitialBalance {
                    address: String::from(TEST_ADDRESS),
                    amount: Uint128::from(TEST_AMOUNT),
                },
            ].to_vec(),
        };

        let (env, info) = mock_env_height("creator", 450, 550);
        let result = instantiate(deps.as_mut(), env, info, instantiate_msg);
        match result {
            Ok(_) => panic!("expected error"),
            Err(ContractError::TickerWrongSymbolFormat {}) => {}
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }
}
