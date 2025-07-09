# SaaS Contract for AMACI

这是一个为AMACI投票系统设计的SaaS (Software as a Service) 合约，主要功能是为operator提供免费的交易服务和round创建服务。

## 主要功能

### 1. Admin/Operator 管理
- **Admin管理**: 合约有一个管理员，可以添加/删除operator
- **Operator管理**: 支持多个operator，每个operator可以使用SaaS服务
- **权限控制**: 严格的权限分离，admin管理operator，operator使用服务

### 2. 资金管理
- **充值功能**: 任何人都可以向SaaS合约充值DORA币
- **提取功能**: 只有admin可以提取合约中的资金
- **余额查询**: 提供合约总余额查询功能

### 3. Fee Grant功能
- **批量Fee Grant**: admin可以为指定地址列表设置fee grant
- **Operator Fee Grant**: admin可以一键为所有operator设置fee grant
- **记录查询**: 提供fee grant记录的查询接口

### 4. AMACI Round创建
- **Operator权限**: 只有注册的operator才能创建AMACI round
- **自动扣费**: 根据round规模自动从SaaS资金中扣取相应费用
- **Registry集成**: 自动调用registry合约创建round

### 5. 消费记录
- **详细记录**: 记录所有操作的详细信息（充值、提取、fee grant、创建round等）
- **按operator查询**: 可以查询特定operator的消费记录
- **分页支持**: 支持分页查询大量记录

## 费用标准

根据AMACI round的规模收取不同费用：
- **小型round** (max_voter ≤ 25, max_option ≤ 5): 20 DORA
- **中型round** (max_voter ≤ 625, max_option ≤ 25): 750 DORA

## 合约接口

### ExecuteMsg

```rust
pub enum ExecuteMsg {
    // 配置管理
    UpdateConfig {
        admin: Option<Addr>,
        registry_contract: Option<Addr>,
        denom: Option<String>,
    },
    
    // Operator管理
    AddOperator { operator: Addr },
    RemoveOperator { operator: Addr },
    
    // 资金管理
    Deposit {},
    Withdraw {
        amount: Uint128,
        recipient: Option<Addr>,
    },
    
    // Fee Grant
    BatchFeegrant {
        recipients: Vec<Addr>,
        amount: Uint128,
    },
    BatchFeeGrantToOperators {
        amount: Uint128,
    },
    
    // 创建AMACI Round
    CreateAmaciRound {
        max_voter: Uint256,
        max_option: Uint256,
        voice_credit_amount: Uint256,
        round_info: RoundInfo,
        voting_time: VotingTime,
        whitelist: Option<WhitelistBase>,
        pre_deactivate_root: Uint256,
        circuit_type: Uint256,
        certification_system: Uint256,
    },
}
```

### QueryMsg

```rust
pub enum QueryMsg {
    Config {},                    // 查询合约配置
    Operators {},                 // 查询所有operator
    IsOperator { address: Addr }, // 检查是否为operator
    Balance {},                   // 查询合约余额
    ConsumptionRecords {          // 查询消费记录
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    FeeGrantRecords {             // 查询fee grant记录
        start_after: Option<Addr>,
        limit: Option<u32>,
    },
    OperatorConsumptionRecords {  // 查询特定operator的消费记录
        operator: Addr,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}
```

## 使用流程

### 1. 合约初始化
```rust
let msg = InstantiateMsg {
    admin: Addr::unchecked("admin_address"),
    registry_contract: Some(Addr::unchecked("registry_contract_address")),
    denom: "peaka".to_string(),
};
```

### 2. 添加Operator
```rust
let msg = ExecuteMsg::AddOperator {
    operator: Addr::unchecked("operator_address"),
};
```

### 3. 充值资金
```rust
let msg = ExecuteMsg::Deposit {};
// 需要在调用时附带DORA币
```

### 4. 设置Fee Grant
```rust
let msg = ExecuteMsg::BatchFeeGrantToOperators {
    amount: Uint128::from(1000000u128), // 1 DORA
};
```

### 5. 创建AMACI Round
```rust
let msg = ExecuteMsg::CreateAmaciRound {
    max_voter: Uint256::from(25u128),
    max_option: Uint256::from(5u128),
    voice_credit_amount: Uint256::from(100u128),
    round_info: RoundInfo {
        title: "Test Round".to_string(),
        description: "A test voting round".to_string(),
        link: "https://example.com".to_string(),
    },
    voting_time: VotingTime {
        start_time: Timestamp::from_seconds(start_time),
        end_time: Timestamp::from_seconds(end_time),
    },
    whitelist: None,
    pre_deactivate_root: Uint256::zero(),
    circuit_type: Uint256::zero(),
    certification_system: Uint256::zero(),
};
```

## 安全特性

1. **权限控制**: 严格的admin和operator权限分离
2. **余额检查**: 所有提取和消费操作都会检查余额
3. **地址验证**: 验证DORA地址格式
4. **完整记录**: 所有操作都有详细的记录和时间戳

## 部署要求

- Rust 1.60+
- CosmWasm 1.1+
- 依赖现有的AMACI合约和Registry合约

这个SaaS合约为AMACI生态系统提供了一个完整的服务层，使operator能够专注于运营投票round，而不需要担心gas费用和round创建费用。 