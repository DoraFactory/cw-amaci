# CreateMaciRound 功能说明

## 概述

我们为 SAAS 合约添加了一个新的 `CreateMaciRound` 功能，该功能允许授权的操作员直接创建新的 MACI 合约实例，而不是通过注册合约来创建 AMACI 轮次。

## 主要特性

### 1. 直接合约部署
- 操作员可以使用预先上传的 MACI 合约代码ID直接部署新的 MACI 合约
- 支持完整的 MACI 初始化参数配置
- 包括 Groth16 和 PLONK 证明系统支持

### 2. 费用管理
- 设置固定的合约部署费用（50 DORA）
- 自动从 SAAS 合约余额中扣除费用
- 记录所有部署活动和费用消耗

### 3. 合约跟踪
- 跟踪所有创建的 MACI 合约信息
- 支持按操作员查询合约
- 提供合约列表和详细信息查询

## 新增的消息类型

### ExecuteMsg::CreateMaciRound

```rust
CreateMaciRound {
    maci_code_id: u64,                              // MACI 合约代码ID
    parameters: MaciParameters,                     // MACI 参数
    coordinator: PubKey,                            // 协调员公钥
    qtr_lib: QuinaryTreeRoot,                       // 五叉树根
    groth16_process_vkey: Option<Groth16VKeyType>,  // Groth16 处理验证密钥
    groth16_tally_vkey: Option<Groth16VKeyType>,    // Groth16 统计验证密钥
    plonk_process_vkey: Option<PlonkVKeyType>,      // PLONK 处理验证密钥
    plonk_tally_vkey: Option<PlonkVKeyType>,        // PLONK 统计验证密钥
    max_vote_options: Uint256,                      // 最大投票选项数
    round_info: RoundInfo,                          // 轮次信息
    voting_time: Option<MaciVotingTime>,            // 投票时间（可选）
    whitelist: Option<Whitelist>,                   // 白名单（可选）
    circuit_type: Uint256,                          // 电路类型 (0: 1p1v | 1: qv)
    certification_system: Uint256,                  // 认证系统 (0: groth16 | 1: plonk)
    admin_override: Option<Addr>,                   // 管理员覆盖（可选）
    label: String,                                  // 合约标签
}
```

## 新增的查询功能

### 1. 查询所有 MACI 合约
```rust
QueryMsg::MaciContracts {
    start_after: Option<u64>,
    limit: Option<u32>,
}
```

### 2. 查询操作员的 MACI 合约
```rust
QueryMsg::OperatorMaciContracts {
    operator: Addr,
    start_after: Option<u64>,
    limit: Option<u32>,
}
```

### 3. 查询特定 MACI 合约
```rust
QueryMsg::MaciContract {
    contract_id: u64,
}
```

## 使用流程

### 1. 准备工作
- 管理员需要添加操作员
- 向 SAAS 合约存入足够的资金（至少 50 DORA）
- 准备 MACI 合约代码ID（需要预先上传）

### 2. 创建 MACI 轮次
- 操作员调用 `CreateMaciRound` 消息
- 提供所有必需的参数
- 系统自动扣除费用并部署合约

### 3. 监控和管理
- 通过查询功能跟踪创建的合约
- 查看费用消耗记录
- 管理操作员权限

## 费用结构

- **部署费用**: 50 DORA （固定费用）
- **余额检查**: 自动验证 SAAS 合约是否有足够余额
- **费用记录**: 所有费用消耗都会被记录在消费记录中

## 权限控制

- **操作员权限**: 只有注册的操作员可以创建 MACI 轮次
- **管理员权限**: 管理员可以添加/删除操作员，管理资金
- **余额保护**: 自动检查余额防止超支

## 测试覆盖

我们提供了完整的测试覆盖，包括：

1. **成功场景**: 正常创建 MACI 轮次
2. **权限测试**: 未授权用户无法创建轮次
3. **余额测试**: 余额不足时无法创建轮次
4. **查询测试**: 验证所有查询功能正常工作

## 与现有功能的区别

| 功能 | CreateAmaciRound | CreateMaciRound |
|------|------------------|-----------------|
| 部署方式 | 通过注册合约 | 直接部署 |
| 费用计算 | 基于电路大小动态 | 固定费用 |
| 管理方式 | 注册合约管理 | SAAS 合约直接管理 |
| 灵活性 | 较低 | 较高 |

这个新功能为用户提供了更灵活的 MACI 合约部署方式，特别适合需要快速部署或自定义配置的场景。 