use cosmwasm_std::{
    entry_point, to_binary, to_vec, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Storage, Uint128,
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use std::convert::TryInto;

use crate::error::ContractError;
use crate::msg::{AllowanceResponse, BalanceResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::Constants;

pub const PREFIX_CONFIG: &[u8] = b"config";
pub const PREFIX_BALANCES: &[u8] = b"balances";
pub const PREFIX_ALLOWANCES: &[u8] = b"allowances";

pub const KEY_CONSTANTS: &[u8] = b"constants";
pub const KEY_TOTAL_SUPPLY: &[u8] = b"total_supply";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // total supply 선언
    let mut total_supply: u128 = 0;
    {
        // balance map 선언
        let mut balance_map = PrefixedStorage::new(deps.storage, PREFIX_BALANCES);

        // balance 다 더해서 total supply 만들기
        for row in msg.initial_balances {
            let raw_amount = row.amount.u128();
            balance_map.set(row.address.as_str().as_bytes(), &raw_amount.to_be_bytes() );
            total_supply += raw_amount;
        }
    }

    // symbol check
    if !is_valid_name(&msg.name) {
        return Err(ContractError::NameWrongFormat{});
    }

    if !is_valid_symbol(&msg.symbol) {
        return Err(ContractError::TickerWrongSymbolFormat{});
    }

    // decimal 자릿수 체크
    if msg.decimals > 18 {
        return Err(ContractError::DecimalsExceeded{});
    }

    // config storage
    let mut config_store = PrefixedStorage::new(deps.storage, PREFIX_CONFIG);
    let constants = to_vec(&Constants{
        name: msg.name,
        decimals: msg.decimals,
        symbol: msg.symbol,
    })?;
    config_store.set(KEY_CONSTANTS, &constants);
    config_store.set(KEY_TOTAL_SUPPLY, &total_supply.to_be_bytes());

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
        ExecuteMsg::Approve { spender, amount } => try_approve(deps, env, info, spender, &amount),
        ExecuteMsg::Transfer { recipient, amount } => {
            try_transfer(deps, env, info, recipient, &amount)
        }
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => try_transfer_from(deps, env, info, owner, recipient, &amount),
        ExecuteMsg::Burn { amount } => try_burn(deps, env, info, &amount),
    }
}

fn try_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: &Uint128,
) -> Result<Response, ContractError> {
    perform_transfer(
        deps.storage,
        &info.sender,
        &deps.api.addr_validate(recipient.as_str())?,
        amount.u128()
    )?;

    Ok(Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
    )
}

fn try_transfer_from(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: &Uint128,
) -> Result<Response, ContractError> {
    let owner_address = deps.api.addr_validate(owner.as_str())?;
    let recipient_address = deps.api.addr_validate(recipient.as_str())?;
    let amount_raw = amount.u128();

    let mut allowance = read_allowance(deps.storage, &owner_address, &info.sender)?;
    if allowance < amount_raw {
        return Err(ContractError::InsufficientAllowance{
            allowance: allowance,
            required: amount_raw,
        });
    }

    allowance -= amount_raw;

    write_allowance(deps.storage, &owner_address, &info.sender, allowance)?;
    perform_transfer(deps.storage, &owner_address, &recipient_address, amount_raw)?;

    Ok(Response::new()
        .add_attribute("action", "transfer_from")
        .add_attribute("spender", &info.sender)
        .add_attribute("sender", owner)
        .add_attribute("recipient", recipient)
    )
}

fn try_approve(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    spender: String,
    amount: &Uint128,
) -> Result<Response, ContractError> {
    let spender_address = deps.api.addr_validate(spender.as_str())?;
    write_allowance(deps.storage, &info.sender, &spender_address, amount.u128())?;
    Ok(Response::new()
        .add_attribute("action", "approve")
        .add_attribute("owner", info.sender)
        .add_attribute("spender", spender)
    )
}

fn try_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: &Uint128,
) -> Result<Response, ContractError> {
    let amount_raw = amount.u128();
    let mut balance = read_balance(deps.storage, &info.sender)?;

    if balance < amount_raw {
        return Err(ContractError::InsufficientFunds{
            balance: balance,
            required: amount_raw,
        });
    }
    balance -= amount_raw;

    let mut balance_store = PrefixedStorage::new(deps.storage, PREFIX_BALANCES);
    balance_store.set(
        info.sender.as_str().as_bytes(),
        &balance.to_be_bytes(),
    );

    let mut config_store = PrefixedStorage::new(deps.storage, PREFIX_CONFIG);
    let total_supply_data = config_store
        .get(KEY_TOTAL_SUPPLY)
        .expect("no total supply data stored");
    let mut total_supply = bytes_to_u128(&total_supply_data)?;
    total_supply -= amount_raw;    

    config_store.set(KEY_TOTAL_SUPPLY, &total_supply.to_be_bytes());

    Ok(Response::new()
        .add_attribute("action", "burn")
        .add_attribute("account", info.sender)
        .add_attribute("amount", amount.to_string())
    )
}

fn perform_transfer(
    store: &mut dyn Storage,
    from: &Addr,
    to: &Addr,
    amount: u128,
) -> Result<(), ContractError> {
    let mut balance_store = PrefixedStorage::new(store, PREFIX_BALANCES);
    let mut from_balance = match balance_store.get(from.as_str().as_bytes()) {
        Some(data) => bytes_to_u128(&data),
        None => Ok(0u128),
    }?;

    if from_balance < amount {
        return Err(ContractError::InsufficientFunds{
            balance: from_balance,
            required: amount,
        });
    }
    from_balance -= amount;
    balance_store.set(from.as_str().as_bytes(), &from_balance.to_be_bytes());

    let mut to_balance = match balance_store.get(to.as_str().as_bytes()) {
        Some(data) => bytes_to_u128(&data),
        None => Ok(0u128),
    }?;
    to_balance += amount;
    balance_store.set(to.as_str().as_bytes(), &to_balance.to_be_bytes());

    Ok(())
}

pub fn bytes_to_u128(data: &[u8]) -> Result<u128, ContractError> {
    match data[0..16].try_into() {
        Ok(bytes) => Ok(u128::from_be_bytes(bytes)),
        Err(_) => Err(ContractError::CorruptedDataFound {}),
    }
}

pub fn read_u128(store: &ReadonlyPrefixedStorage, key: &Addr) -> Result<u128, ContractError> {
    let result = store.get(key.as_str().as_bytes());
    match result {
        Some(data) => bytes_to_u128(&data),
        None => Ok(0u128),
    }
}

fn read_balance(store: &dyn Storage, owner: &Addr) -> Result<u128, ContractError> {
    let balance_store = ReadonlyPrefixedStorage::new(store, PREFIX_BALANCES);
    read_u128(&balance_store, owner)
}

fn read_allowance(
    store: &dyn Storage,
    owner: &Addr,
    spender: &Addr,
) -> Result<u128, ContractError> {
    let owner_store =
        ReadonlyPrefixedStorage::multilevel(store, &[PREFIX_ALLOWANCES, owner.as_str().as_bytes()]);
    read_u128(&owner_store, spender)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Balance { address } => {
            let address_key = deps.api.addr_validate(&address)?;
            let balance = read_balance(deps.storage, &address_key)?;
            let out = to_binary(&BalanceResponse {
                balance: Uint128::from(balance),
            })?;
            Ok(out)
        }
        QueryMsg::Allowance { owner, spender } => {
            let owner_key = deps.api.addr_validate(&owner)?;
            let spender_key = deps.api.addr_validate(&spender)?;
            let allowance = read_allowance(deps.storage, &owner_key, &spender_key)?;
            let out = to_binary(&AllowanceResponse {
                allowance: Uint128::from(allowance),
            })?;
            Ok(out)
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn write_allowance(
    store: &mut dyn Storage,
    owner: &Addr,
    spender: &Addr,
    amount: u128,
) -> StdResult<()> {
    let mut owner_store =
        PrefixedStorage::multilevel(store, &[PREFIX_ALLOWANCES, owner.as_str().as_bytes()]);
    owner_store.set(spender.as_str().as_bytes(), &amount.to_be_bytes());
    Ok(())
}

fn is_valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.len() < 3 || bytes.len() > 30 {
        return false;
    }
    true
}

fn is_valid_symbol(symbol: &str) -> bool {
    let bytes = symbol.as_bytes();
    if bytes.len() < 3 || bytes.len() > 6 {
        return false;
    }
    for byte in bytes.iter() {
        if *byte < 65 || *byte > 90 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

}
