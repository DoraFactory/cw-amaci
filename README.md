# cw-amaci
CosmWasm A-MACI

## 项目概述

这是一个基于CosmWasm的A-MACI (Anonymous Minimum Anti-Collusion Infrastructure) 投票系统，包含三个主要合约：

### 合约组件

1. **AMACI合约** (`contracts/amaci/`)
   - 核心投票合约，实现匿名投票和防共谋机制
   - 支持零知识证明验证
   - 提供完整的投票生命周期管理

2. **Registry合约** (`contracts/registry/`)
   - 投票轮次注册和管理合约
   - 处理operator管理和验证
   - 根据投票规模收取相应费用

3. **SaaS合约** (`contracts/saas/`) - 🆕 新增
   - 为AMACI生态系统提供SaaS服务
   - 支持operator管理和资金池管理
   - 提供批量fee grant功能
   - 自动化投票轮次创建和付费

## SaaS合约特性

- ✅ **Admin/Operator管理**: 支持多operator管理体系
- ✅ **资金管理**: 任何人可充值，只有admin可提取
- ✅ **批量Fee Grant**: 支持给operator或指定地址列表批量设置fee grant
- ✅ **自动付费创建Round**: operator可使用SaaS资金免费创建投票轮次
- ✅ **消费记录**: 完整的操作和消费记录，便于记账
- ✅ **权限控制**: 严格的权限分离和安全检查

## 快速开始

### 编译

```bash
# 编译所有合约
cargo build

# 检查代码
cargo check --all

# 运行测试
cargo test
```

### 部署顺序

1. 首先部署 AMACI 合约
2. 然后部署 Registry 合约
3. 最后部署 SaaS 合约，并设置 Registry 合约地址

## 使用场景

SaaS合约主要为以下场景设计：

1. **DAO治理**: 为DAO成员提供免费的投票服务
2. **企业投票**: 企业内部决策投票，由企业承担gas费用
3. **社区治理**: 社区管理者为用户提供免费投票体验
4. **投票即服务**: 为第三方应用提供投票基础设施服务

## 费用标准

- 小型投票 (≤25人, ≤5选项): 20 DORA
- 中型投票 (≤625人, ≤25选项): 750 DORA

## 贡献

欢迎提交Issue和Pull Request来改进项目。

## 许可证

本项目采用适当的开源许可证。
