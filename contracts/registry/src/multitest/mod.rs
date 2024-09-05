#[cfg(test)]
mod tests;

use anyhow::Result as AnyResult;

use crate::{
    contract::{execute, instantiate, query},
    msg::*,
};
use cosmwasm_std::{Addr, Coin, StdResult, Timestamp, Uint128, Uint256};
use cw_amaci::{
    msg::InstantiationData,
    state::{PubKey, RoundInfo, VotingTime},
};
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";
pub const DORA_DEMON: &str = "peaka";
pub const DORA_DECIMALS: u8 = 18;
pub const MIN_DEPOSIT_AMOUNT: u128 = 20u128;
pub const SLASH_AMOUNT: u128 = 1u128; // only 1, 2 (admin amount is not enough)
use num_bigint::BigUint;

pub fn uint256_from_decimal_string(decimal_string: &str) -> Uint256 {
    assert!(
        decimal_string.len() <= 77,
        "the decimal length can't abrove 77"
    );

    let decimal_number = BigUint::parse_bytes(decimal_string.as_bytes(), 10)
        .expect("Failed to parse decimal string");

    let byte_array = decimal_number.to_bytes_be();

    let hex_string = hex::encode(byte_array);
    uint256_from_hex_string(&hex_string)
}

pub fn uint256_from_hex_string(hex_string: &str) -> Uint256 {
    let padded_hex_string = if hex_string.len() < 64 {
        let padding_length = 64 - hex_string.len();
        format!("{:0>width$}{}", "", hex_string, width = padding_length)
    } else {
        hex_string.to_string()
    };

    let res = hex_to_decimal(&padded_hex_string);
    Uint256::from_be_bytes(res)
}

pub fn hex_to_decimal(hex_bytes: &str) -> [u8; 32] {
    let bytes = hex::decode(hex_bytes).unwrap_or_else(|_| vec![]);
    let decimal_values: Vec<u8> = bytes.iter().cloned().collect();

    let mut array: [u8; 32] = [0; 32];

    if decimal_values.len() >= 32 {
        array.copy_from_slice(&decimal_values[..32]);
    } else {
        array[..decimal_values.len()].copy_from_slice(&decimal_values);
    }

    array
}

#[derive(Clone, Debug, Copy)]
pub struct AmaciRegistryCodeId(u64);

impl AmaciRegistryCodeId {
    pub fn store_code(app: &mut App) -> Self {
        let contract =
            ContractWrapper::new(execute, instantiate, query).with_reply(cw_amaci::contract::reply);
        let code_id = app.store_code(Box::new(contract));
        Self(code_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn instantiate(
        self,
        app: &mut App,
        sender: Addr,
        label: &str,
    ) -> AnyResult<AmaciRegistryContract> {
        AmaciRegistryContract::instantiate(app, self, sender, operator(), label)
    }
}

impl From<AmaciRegistryCodeId> for u64 {
    fn from(code_id: AmaciRegistryCodeId) -> Self {
        code_id.0
    }
}

#[derive(Debug, Clone)]
pub struct AmaciRegistryContract(Addr);

// implement the contract real function, e.g. instantiate, functions in exec, query modules
impl AmaciRegistryContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    #[allow(clippy::too_many_arguments)]
    #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: AmaciRegistryCodeId,
        sender: Addr,
        operator: Addr,
        label: &str,
    ) -> AnyResult<Self> {
        let init_msg = InstantiateMsg {
            denom: DORA_DEMON.to_string(),
            min_deposit_amount: Uint128::from(MIN_DEPOSIT_AMOUNT),
            slash_amount: Uint128::from(SLASH_AMOUNT),
            admin: sender.clone(),
            operator,
        };
        app.instantiate_contract(code_id.0, sender, &init_msg, &[], label, None)
            .map(Self::from)
    }

    #[track_caller]
    pub fn register(
        &self,
        app: &mut App,
        sender: Addr,
        pubkey: PubKey,
        sent: &[Coin],
    ) -> AnyResult<AppResponse> {
        app.execute_contract(sender, self.addr(), &ExecuteMsg::Register { pubkey }, sent)
    }

    #[track_caller]
    pub fn deregister(&self, app: &mut App, sender: Addr) -> AnyResult<AppResponse> {
        app.execute_contract(sender, self.addr(), &ExecuteMsg::Deregister {}, &[])
    }

    #[track_caller]
    pub fn create_round(
        &self,
        app: &mut App,
        sender: Addr,
        amaci_code_id: u64,
        operator: Addr,
        // ) -> AnyResult<Option<InstantiationData>> {
    ) -> AnyResult<AppResponse> {
        let msg = ExecuteMsg::CreateRound {
            amaci_code_id,
            operator,
            max_voter: Uint256::from_u128(5u128),
            max_option: Uint256::from_u128(5u128),
            voice_credit_amount: Uint256::from_u128(30u128),
            round_info: RoundInfo {
                title: "".to_string(),
                description: "".to_string(),
                link: "".to_string(),
            },
            voting_time: VotingTime {
                start_time: Timestamp::from_nanos(1571797424879000000),
                end_time: Timestamp::from_nanos(1571797429879300000),
            },
            whitelist: None,
            pre_deactivate_root: Uint256::from_u128(0u128),
        };

        app.execute_contract(sender, self.addr(), &msg, &[])
        // app.execute_contract(sender, self.addr(), &msg, &[])?;
        // Ok(None)
    }

    // #[track_caller]
    // pub fn upload_deactivate_message(
    //     &self,
    //     app: &mut App,
    //     sender: Addr,
    //     contract_address: Addr,
    //     deactivate_message: Vec<Vec<Uint256>>,
    // ) -> AnyResult<AppResponse> {
    //     app.execute_contract(
    //         sender,
    //         self.addr(),
    //         &ExecuteMsg::UploadDeactivateMessage {
    //             contract_address,
    //             deactivate_message,
    //         },
    //         &[],
    //     )
    // }

    #[track_caller]
    pub fn change_params(
        &self,
        app: &mut App,
        sender: Addr,
        min_deposit_amount: Uint128,
        slash_amount: Uint128,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::ChangeParams {
                min_deposit_amount,
                slash_amount,
            },
            &[],
        )
    }

    #[track_caller]
    pub fn change_operator(
        &self,
        app: &mut App,
        sender: Addr,
        address: Addr,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::ChangeOperator { address },
            &[],
        )
    }

    pub fn get_admin(&self, app: &App) -> StdResult<AdminResponse> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::Admin {})
    }

    pub fn is_maci_operator(&self, app: &App, address: Addr) -> StdResult<bool> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::IsMaciOperator { address })
    }

    pub fn get_deactivate_message(
        &self,
        app: &App,
        contract_address: Addr,
    ) -> StdResult<Vec<Vec<String>>> {
        app.wrap().query_wasm_smart(
            self.addr(),
            &QueryMsg::GetMaciDeactivate { contract_address },
        )
    }

    pub fn get_maci_operator(&self, app: &App, contract_address: Addr) -> StdResult<Addr> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::GetMaciOperator { contract_address })
    }

    pub fn get_total(&self, app: &App) -> StdResult<u128> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::GetTotal {})
    }

    pub fn balance_of(&self, app: &App, address: String, denom: String) -> StdResult<Coin> {
        app.wrap().query_balance(address, denom)
    }
}

impl From<Addr> for AmaciRegistryContract {
    fn from(value: Addr) -> Self {
        Self(value)
    }
}

pub fn user1() -> Addr {
    Addr::unchecked("user1")
}

pub fn user2() -> Addr {
    Addr::unchecked("user2")
}

pub fn user3() -> Addr {
    Addr::unchecked("user3")
}

pub fn user4() -> Addr {
    Addr::unchecked("user4")
}

pub fn user5() -> Addr {
    Addr::unchecked("user5")
}

pub fn owner() -> Addr {
    Addr::unchecked("dora1t58t7azqzq26406uwehgnfekal5kzym3m9lz4k")
}

pub fn operator() -> Addr {
    Addr::unchecked("dora1qdagdkg9me4253h9qyvx83sd4gpta6rzh2fa0j")
}

pub fn operator2() -> Addr {
    Addr::unchecked("dora1tuu2qpj0ytj2k7fta7u5fwruzeyyfqj5q4vq23")
}

pub fn contract_address() -> Addr {
    Addr::unchecked("dora1smdzpfsy48kmkzmm4m9hsg4850czdvfncxyxp6d4h3j7qv3m4v0s0530a6")
}
