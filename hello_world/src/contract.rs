use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

// use crate::error::ContractError;
use crate::msg::{CurrNameResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{writable_name, name_read, State};
use crate::error::ContractError;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        name: msg.register,
    };

    writable_name(deps.storage).save(&state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register { name } => execute_register(deps, env, info, name),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Hello {} => query_hello(deps, _env),
    }
}

pub fn execute_register(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    name: String,
) -> Result<Response, ContractError> {
    // we only need to check here - at point of registration
    match writable_name(deps.storage).may_load()? {
        Some(curr_state) => {
            if curr_state.name == name {
                return Err(ContractError::NameTaken{name: curr_state.name});
            }
        }
        None => {
            return Err(ContractError::NameNotExists{ name });
        }
    };

    // name is available
    let new_record = State{
        name: name,
    };
    writable_name(deps.storage).save(&new_record)?;

    Ok(Response::default())
}

fn query_hello(deps: Deps, _env: Env) -> StdResult<Binary> {
    match name_read(deps.storage).may_load()? {
        Some(curr_state) => to_binary( &CurrNameResponse{ name: format!("Hello, {}", curr_state.name)}),
        None => to_binary( &CurrNameResponse{ name: String::from("") } ),
    }
}
