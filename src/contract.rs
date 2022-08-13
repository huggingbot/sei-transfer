use crate::state::{Config, CONFIG, WITHDRAWABLE};
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, Uint64,
};
#[cfg(not(feature = "library"))]
use cw2::set_contract_version;
use cw20::{Cw20Contract, Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{
    Cw20ContractResponse, ExecuteMsg, GetFeeBasisPointResponse, InstantiateMsg, OwnerResponse,
    QueryMsg, ReceiveMsg, WithdrawableResponse,
};

const CONTRACT_NAME: &str = "crates.io:sei-transfer";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = _deps.api.addr_validate(&_msg.owner)?;
    let config = Config {
        owner,
        cw20_addr: _deps.api.addr_validate(&_msg.cw20_addr)?,
        cw20_decimals: _msg.cw20_decimals,
        fee_basis_point: _msg.fee_basis_point,
    };
    CONFIG.save(_deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match _msg {
        ExecuteMsg::Receive(msg) => execute_receive(_deps, _info, msg),
        ExecuteMsg::Withdraw { amount } => execute_withdraw(_deps, _info, amount),
        ExecuteMsg::SetFeeBasisPoint { fee_basis_point } => {
            execute_set_fee_basis_point(_deps, _info, fee_basis_point)
        }
    }
}

fn execute_receive(
    _deps: DepsMut,
    _info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(_deps.storage)?;
    if config.cw20_addr != _info.sender {
        return Err(ContractError::Unauthorized {});
    }
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    match msg {
        ReceiveMsg::SetReceivers { addr1, addr2 } => {
            receive_set_receivers(_deps, addr1, addr2, wrapper.amount)
        }
    }
}

fn receive_set_receivers(
    _deps: DepsMut,
    addr1: String,
    addr2: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let addr1 = _deps.api.addr_validate(&addr1)?;
    let addr2 = _deps.api.addr_validate(&addr2)?;

    let fee = calculate_fee(_deps.as_ref(), amount)?;
    let amount_after_fee = amount.checked_sub(fee)?;
    let addr1_add = amount_after_fee.checked_div(Uint128::new(2))?;
    let addr2_add = amount_after_fee.checked_sub(addr1_add)?;

    let addr1_amount = WITHDRAWABLE.update(
        _deps.storage,
        addr1.clone(),
        |withdrawable| -> StdResult<_> {
            Ok(withdrawable.unwrap_or_default().checked_add(addr1_add)?)
        },
    )?;

    let addr2_amount = WITHDRAWABLE.update(
        _deps.storage,
        addr2.clone(),
        |withdrawable| -> StdResult<_> {
            Ok(withdrawable.unwrap_or_default().checked_add(addr2_add)?)
        },
    )?;

    Ok(Response::new()
        .add_attribute(format!("{}", addr1), addr1_amount)
        .add_attribute(format!("{}", addr2), addr2_amount)
        .add_attribute("fee", fee))
}

fn calculate_fee(deps: Deps, amount: Uint128) -> Result<Uint128, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let factor = Uint128::new(10).checked_pow(config.cw20_decimals)?;
    let fee_basis_point_atomic = Uint128::from(config.fee_basis_point)
        .checked_mul(factor)?
        .checked_div(Uint128::new(10).checked_pow(4)?)?;
    let fee = amount
        .checked_mul(fee_basis_point_atomic)?
        .checked_div(factor)?;
    Ok(fee)
}

fn execute_withdraw(
    _deps: DepsMut,
    _info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(_deps.storage)?;
    let sender = _info.sender;
    let withdrawable = WITHDRAWABLE.load(_deps.storage, sender.clone())?;
    let leftover = withdrawable.checked_sub(amount)?;

    if leftover.eq(&Uint128::zero()) {
        WITHDRAWABLE.remove(_deps.storage, sender.clone());
    } else {
        WITHDRAWABLE.save(_deps.storage, sender.clone(), &leftover)?;
    }

    let cw20 = Cw20Contract(config.cw20_addr);
    let msg = cw20.call(Cw20ExecuteMsg::Transfer {
        recipient: sender.to_string(),
        amount,
    })?;

    Ok(Response::new()
        .add_attribute(format!("{}", sender.clone()), leftover)
        .add_message(msg))
}

fn execute_set_fee_basis_point(
    _deps: DepsMut,
    _info: MessageInfo,
    fee_basis_point: Uint64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(_deps.storage)?;
    if config.owner != _info.sender {
        return Err(ContractError::Unauthorized {});
    }
    CONFIG.update(_deps.storage, |config| -> StdResult<_> {
        Ok(Config {
            owner: config.owner,
            cw20_addr: config.cw20_addr,
            cw20_decimals: config.cw20_decimals,
            fee_basis_point,
        })
    })?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    match _msg {
        QueryMsg::GetOwner {} => to_binary(&query_owner(_deps)?),
        QueryMsg::GetCw20Contract {} => to_binary(&query_cw20_contract(_deps)?),
        QueryMsg::GetWithdrawable { addr } => to_binary(&query_withdrawable(_deps, addr)?),
        QueryMsg::GetFeeBasisPoint {} => to_binary(&query_fee_basis_point(_deps)?),
    }
}

fn query_owner(_deps: Deps) -> StdResult<OwnerResponse> {
    let config = CONFIG.load(_deps.storage)?;
    Ok(OwnerResponse {
        owner: config.owner.into_string(),
    })
}

fn query_cw20_contract(_deps: Deps) -> StdResult<Cw20ContractResponse> {
    let config = CONFIG.load(_deps.storage)?;
    Ok(Cw20ContractResponse {
        contract: config.cw20_addr.into_string(),
    })
}

fn query_withdrawable(_deps: Deps, addr: String) -> StdResult<WithdrawableResponse> {
    let addr = _deps.api.addr_validate(&addr)?;
    let withdrawable = WITHDRAWABLE.load(_deps.storage, addr)?;
    Ok(WithdrawableResponse { withdrawable })
}

fn query_fee_basis_point(_deps: Deps) -> StdResult<GetFeeBasisPointResponse> {
    let config = CONFIG.load(_deps.storage)?;
    Ok(GetFeeBasisPointResponse {
        fee_basis_point: config.fee_basis_point,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{Attribute, CosmosMsg, StdError, WasmMsg};

    const OWNER: &str = "owner";
    const USER1: &str = "user1";
    const USER2: &str = "user2";
    const USER3: &str = "user3";
    const CW20_DECIMALS: u32 = 18;
    const FEE_BASIS_POINT: u64 = 100;

    fn atomize(number: u128) -> Uint128 {
        let factor = Uint128::new(10).checked_pow(CW20_DECIMALS).unwrap();
        Uint128::from(number).checked_mul(factor).unwrap()
    }

    fn default_instantiate(deps: DepsMut) {
        let info = mock_info(OWNER, &[]);
        let msg = InstantiateMsg {
            owner: OWNER.to_string(),
            cw20_addr: MOCK_CONTRACT_ADDR.to_string(),
            cw20_decimals: CW20_DECIMALS,
            fee_basis_point: Uint64::new(FEE_BASIS_POINT),
        };

        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);
    }

    fn receive(
        deps: DepsMut,
        cw20_contract: &str,
        sender: String,
        addr1: String,
        addr2: String,
        amount: Uint128,
    ) -> Result<(), ContractError> {
        let info = mock_info(cw20_contract, &[]);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: sender.clone(),
            amount,
            msg: to_binary(&ReceiveMsg::SetReceivers {
                addr1: addr1.clone(),
                addr2: addr2.clone(),
            })
            .unwrap(),
        });

        let fee = calculate_fee(deps.as_ref(), amount)?;
        let amount_after_fee = amount.checked_sub(fee)?;
        let addr1_add = amount_after_fee.checked_div(Uint128::new(2))?;
        let addr2_add = amount_after_fee.checked_sub(addr1_add)?;

        let res = execute(deps, mock_env(), info, msg)?;

        assert_eq!(
            res.attributes,
            vec![
                Attribute {
                    key: addr1.clone(),
                    value: addr1_add.to_string()
                },
                Attribute {
                    key: addr2.clone(),
                    value: addr2_add.to_string()
                },
                Attribute {
                    key: "fee".to_string(),
                    value: fee.to_string()
                }
            ]
        );
        Ok(())
    }

    fn withdraw(deps: DepsMut, withdrawer: &str, amount: Uint128) -> Result<(), ContractError> {
        let withdrawable = query_withdrawable(deps.as_ref(), withdrawer.to_string())?;

        let info = mock_info(withdrawer, &[]);
        let msg = ExecuteMsg::Withdraw { amount };
        let res = execute(deps, mock_env(), info, msg)?;

        assert_eq!(
            res.attributes,
            vec![Attribute {
                key: withdrawer.to_string(),
                value: withdrawable.checked_sub(amount).unwrap().to_string()
            }]
        );
        assert_eq!(res.messages.len(), 1);
        let msg = res.messages[0].clone().msg;

        assert_eq!(
            msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: withdrawer.to_string(),
                    amount,
                })
                .unwrap(),
                funds: vec![]
            })
        );

        Ok(())
    }

    fn query_withdrawable(deps: Deps, addr: String) -> StdResult<Uint128> {
        let msg = QueryMsg::GetWithdrawable { addr };
        let res = query(deps, mock_env(), msg)?;
        let withdrawable_res: WithdrawableResponse = from_binary(&res).unwrap();
        Ok(withdrawable_res.withdrawable)
    }

    fn query_fee_basis_point(deps: Deps) -> StdResult<Uint64> {
        let msg = QueryMsg::GetFeeBasisPoint {};
        let res = query(deps, mock_env(), msg)?;
        let fee_basis_point_res: GetFeeBasisPointResponse = from_binary(&res).unwrap();
        Ok(fee_basis_point_res.fee_basis_point)
    }

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut());

        let msg = QueryMsg::GetOwner {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let owner_res: OwnerResponse = from_binary(&res).unwrap();

        let msg = QueryMsg::GetCw20Contract {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let cw20_contract_res: Cw20ContractResponse = from_binary(&res).unwrap();

        assert_eq!(owner_res.owner, OWNER);
        assert_eq!(cw20_contract_res.contract, MOCK_CONTRACT_ADDR);
    }

    #[test]
    fn set_fee_basis_point() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut());

        let fee_basis_point = query_fee_basis_point(deps.as_ref()).unwrap();
        assert_eq!(fee_basis_point, Uint64::new(100));

        let info = mock_info(OWNER, &[]);
        let msg = ExecuteMsg::SetFeeBasisPoint {
            fee_basis_point: Uint64::new(200),
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let fee_basis_point = query_fee_basis_point(deps.as_ref()).unwrap();
        assert_eq!(fee_basis_point, Uint64::new(200));
    }

    #[test]
    fn unauthorised_set_fee_basis_point() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut());

        let fee_basis_point = query_fee_basis_point(deps.as_ref()).unwrap();
        assert_eq!(fee_basis_point, Uint64::new(100));

        let info = mock_info(USER1, &[]);
        let msg = ExecuteMsg::SetFeeBasisPoint {
            fee_basis_point: Uint64::new(200),
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn send_tokens() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut());

        let user1 = USER1.to_string();
        let user2 = USER2.to_string();
        let user3 = USER3.to_string();

        let err = query_withdrawable(deps.as_ref(), user2.clone()).unwrap_err();
        assert_eq!(
            err,
            StdError::NotFound {
                kind: "cosmwasm_std::math::uint128::Uint128".to_string()
            }
        );

        let err = query_withdrawable(deps.as_ref(), user3.clone()).unwrap_err();
        assert_eq!(
            err,
            StdError::NotFound {
                kind: "cosmwasm_std::math::uint128::Uint128".to_string()
            }
        );

        receive(
            deps.as_mut(),
            MOCK_CONTRACT_ADDR,
            user1.clone(),
            user2.clone(),
            user3.clone(),
            atomize(1_000),
        )
        .unwrap();

        let withdrawable = query_withdrawable(deps.as_ref(), user2.clone()).unwrap();
        assert_eq!(withdrawable, atomize(495));

        let withdrawable = query_withdrawable(deps.as_ref(), user3.clone()).unwrap();
        assert_eq!(withdrawable, atomize(495));
    }

    #[test]
    fn unauthorised_send_tokens() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut());

        let user1 = USER1.to_string();
        let user2 = USER2.to_string();
        let user3 = USER3.to_string();

        let err = receive(
            deps.as_mut(),
            "random_contract",
            user1.clone(),
            user2.clone(),
            user3.clone(),
            atomize(1_000),
        )
        .unwrap_err();

        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn no_withdrawable_tokens() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut());

        let err = withdraw(deps.as_mut(), USER2, atomize(500)).unwrap_err();
        assert_eq!(
            err,
            ContractError::Std(StdError::NotFound {
                kind: "cosmwasm_std::math::uint128::Uint128".to_string()
            })
        );
    }

    #[test]
    fn withdraw_some_tokens() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut());

        let user1 = USER1.to_string();
        let user2 = USER2.to_string();
        let user3 = USER3.to_string();

        receive(
            deps.as_mut(),
            MOCK_CONTRACT_ADDR,
            user1.clone(),
            user2.clone(),
            user3.clone(),
            atomize(1_000),
        )
        .unwrap();

        let withdrawable = query_withdrawable(deps.as_ref(), user2.clone()).unwrap();
        assert_eq!(withdrawable, atomize(495));

        withdraw(deps.as_mut(), USER2, atomize(300)).unwrap();

        let withdrawable = query_withdrawable(deps.as_ref(), user2.clone()).unwrap();
        assert_eq!(withdrawable, atomize(195));
    }

    #[test]
    fn withdraw_all_tokens() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut());

        let user1 = USER1.to_string();
        let user2 = USER2.to_string();
        let user3 = USER3.to_string();

        receive(
            deps.as_mut(),
            MOCK_CONTRACT_ADDR,
            user1.clone(),
            user2.clone(),
            user3.clone(),
            atomize(1_000),
        )
        .unwrap();

        let withdrawable = query_withdrawable(deps.as_ref(), user2.clone()).unwrap();
        assert_eq!(withdrawable, atomize(495));

        withdraw(deps.as_mut(), USER2, atomize(495)).unwrap();

        let err = query_withdrawable(deps.as_ref(), user2.clone()).unwrap_err();
        assert_eq!(
            err,
            StdError::NotFound {
                kind: "cosmwasm_std::math::uint128::Uint128".to_string()
            }
        );
    }
}
