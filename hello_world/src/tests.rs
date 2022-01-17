use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, coins, from_binary, Coin, Deps, DepsMut};

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, CurrNameResponse};
use crate::state::State;

#[test]
fn test_check_init_proper_config() {
    let creater = "bryan";

    let msg  = InstantiateMsg {
        register: String::from(creater),
    };
    let info = mock_info("creator", &coins(2, "token"));

    let mut deps = mock_dependencies(&[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg)
            .expect("contract successfully handles InstantiateMsg");

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Hello{},
    )
    .unwrap();

    match from_binary::<CurrNameResponse>(&res) {
        Ok(obj) => {
            println!("{}", obj.name);
            assert_eq!(format!("Hello, {}", creater), obj.name);
            return;
        },
        Err(err) => {
            panic!("Unexpected error: {:?}", err);
        }
    };
}
