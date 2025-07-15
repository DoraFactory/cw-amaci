#[cfg(test)]
mod tests;

use anyhow::Result as AnyResult;
use cosmwasm_std::{Addr, Coin, StdResult, Timestamp, Uint128};
use cw_amaci::state::RoundInfo;
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};

use crate::{
    contract::{execute, instantiate, migrate, query, reply},
    msg::*,
    state::{Config, FeeGrantRecord, OperatorInfo},
};

pub const DORA_DEMON: &str = "peaka";

#[derive(Clone, Debug, Copy)]
pub struct SaasCodeId(u64);

impl SaasCodeId {
    pub fn store_code(app: &mut App) -> Self {
        let contract = ContractWrapper::new(execute, instantiate, query)
            .with_migrate(migrate)
            .with_reply(reply);
        let code_id = app.store_code(Box::new(contract));
        Self(code_id)
    }

    pub fn instantiate(
        self,
        app: &mut App,
        sender: Addr,
        admin: Addr,
        registry_contract: Option<Addr>,
        denom: String,
        label: &str,
    ) -> AnyResult<SaasContract> {
        SaasContract::instantiate(app, self, sender, admin, registry_contract, denom, label)
    }
}

impl From<SaasCodeId> for u64 {
    fn from(code_id: SaasCodeId) -> Self {
        code_id.0
    }
}

#[derive(Clone, Debug)]
pub struct SaasContract(Addr);

impl SaasContract {
    fn addr(&self) -> Addr {
        self.0.clone()
    }

    #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: SaasCodeId,
        sender: Addr,
        admin: Addr,
        registry_contract: Option<Addr>,
        denom: String,
        label: &str,
    ) -> AnyResult<Self> {
        let init_msg = InstantiateMsg {
            admin,
            registry_contract,
            denom,
        };

        app.instantiate_contract(code_id.0, sender, &init_msg, &[], label, None)
            .map(Self)
    }

    #[track_caller]
    pub fn update_config(
        &self,
        app: &mut App,
        sender: Addr,
        admin: Option<Addr>,
        registry_contract: Option<Addr>,
        denom: Option<String>,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::UpdateConfig {
                admin,
                registry_contract,
                denom,
            },
            &[],
        )
    }

    #[track_caller]
    pub fn add_operator(
        &self,
        app: &mut App,
        sender: Addr,
        operator: Addr,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::AddOperator { operator },
            &[],
        )
    }

    #[track_caller]
    pub fn remove_operator(
        &self,
        app: &mut App,
        sender: Addr,
        operator: Addr,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::RemoveOperator { operator },
            &[],
        )
    }

    #[track_caller]
    pub fn deposit(&self, app: &mut App, sender: Addr, funds: &[Coin]) -> AnyResult<AppResponse> {
        app.execute_contract(sender, self.addr(), &ExecuteMsg::Deposit {}, funds)
    }

    #[track_caller]
    pub fn withdraw(
        &self,
        app: &mut App,
        sender: Addr,
        amount: Uint128,
        recipient: Option<Addr>,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::Withdraw { amount, recipient },
            &[],
        )
    }

    #[track_caller]
    pub fn batch_feegrant(
        &self,
        app: &mut App,
        sender: Addr,
        recipients: Vec<Addr>,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::BatchFeegrant { recipients, amount },
            &[],
        )
    }

    #[track_caller]
    pub fn batch_feegrant_to_operators(
        &self,
        app: &mut App,
        sender: Addr,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        app.execute_contract(
            sender,
            self.addr(),
            &ExecuteMsg::BatchFeeGrantToOperators { amount },
            &[],
        )
    }

    // Query methods
    pub fn query_config(&self, app: &App) -> StdResult<Config> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::Config {})
    }

    pub fn query_operators(&self, app: &App) -> StdResult<Vec<OperatorInfo>> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::Operators {})
    }

    pub fn query_is_operator(&self, app: &App, address: Addr) -> StdResult<bool> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::IsOperator { address })
    }

    pub fn query_balance(&self, app: &App) -> StdResult<Uint128> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::Balance {})
    }

    pub fn query_feegrant_records(
        &self,
        app: &App,
        start_after: Option<Addr>,
        limit: Option<u32>,
    ) -> StdResult<Vec<FeeGrantRecord>> {
        app.wrap().query_wasm_smart(
            self.addr(),
            &QueryMsg::FeeGrantRecords { start_after, limit },
        )
    }

    pub fn query_maci_contracts(
        &self,
        app: &App,
        start_after: Option<u64>,
        limit: Option<u32>,
    ) -> StdResult<Vec<crate::state::MaciContractInfo>> {
        app.wrap()
            .query_wasm_smart(self.addr(), &QueryMsg::MaciContracts { start_after, limit })
    }

    pub fn balance_of(&self, app: &App, address: String, denom: String) -> StdResult<Coin> {
        app.wrap().query_balance(address, denom)
    }
}

impl From<Addr> for SaasContract {
    fn from(value: Addr) -> Self {
        Self(value)
    }
}

// Helper functions for creating test addresses
pub fn admin() -> Addr {
    Addr::unchecked("admin")
}

pub fn creator() -> Addr {
    Addr::unchecked("creator")
}

pub fn operator1() -> Addr {
    Addr::unchecked("operator1")
}

pub fn operator2() -> Addr {
    Addr::unchecked("operator2")
}

pub fn operator3() -> Addr {
    Addr::unchecked("operator3")
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

pub fn mock_registry_contract() -> Addr {
    Addr::unchecked("registry_contract")
}

// Helper function to create test round info
pub fn test_round_info() -> RoundInfo {
    RoundInfo {
        title: "Test Round".to_string(),
        description: "A test voting round".to_string(),
        link: "https://example.com".to_string(),
    }
}

// Helper function to create test voting time (for legacy AMACI tests)
pub fn test_voting_time() -> cw_amaci::state::VotingTime {
    cw_amaci::state::VotingTime {
        start_time: Timestamp::from_seconds(1640995200), // 2022-01-01
        end_time: Timestamp::from_seconds(1641081600),   // 2022-01-02
    }
}

// Helper function to setup a real registry contract for integration testing (legacy)
pub fn setup_registry_contract(
    app: &mut App,
) -> cw_amaci_registry::multitest::AmaciRegistryContract {
    use cw_amaci::multitest::MaciCodeId;
    use cw_amaci_registry::multitest::AmaciRegistryCodeId;

    let registry_code_id = AmaciRegistryCodeId::store_code(app);
    let amaci_code_id = MaciCodeId::store_default_code(app);

    registry_code_id
        .instantiate(app, creator(), amaci_code_id.id(), "Test Registry")
        .unwrap()
}
