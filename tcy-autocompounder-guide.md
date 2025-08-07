
# TCY Auto-Compounder Integration Guide

The TCY auto-compounder contract is live on Rujira and ready for integration. It allows users to:

1. Deposit TCY and receive `sTCY` (auto-compounding mode).
2. Redeem `sTCY` for TCY.
3. Query the current status of the contract.

You can see the official implementation here:  
👉 https://tcy.thorchain.org/manage

We **strongly recommend** you only support the **auto-compound** flow (Liquid mode), just like the official site. The contract technically supports non-compounding mode too, but for simplicity and clarity to users, stick to the compounding path unless you have a strong reason not to.

---

## 🔌 Contract Address

```
thor1z7ejlk5wk2pxh9nfwjzkkdnrq4p2f5rjcpudltv0gh282dwfz6nq9g2cr0
```

---

## 🟢 Deposit (Bond) – Get sTCY

This operation deposits TCY and mints `sTCY` (liquid staking token) back to the user. Rewards are auto-compounded.

### ⚙️ ExecuteMsg:
```json
{
  "liquid": {
    "bond": {}
  }
}
```

### 📦 CLI Example:
```bash
thornode tx wasm execute thor1z7ejlk5wk2pxh9nfwjzkkdnrq4p2f5rjcpudltv0gh282dwfz6nq9g2cr0 '{"liquid": {"bond": {}}}'   --amount 100000000tcy   --from <wallet>   --gas auto --gas-adjustment 2   --chain-id thorchain-1   --node https://rpc.ninerealms.com -y
```

After successful execution, the user will receive `x/staking-tcy` in return.

---

## 🔴 Withdraw (Unbond) – Redeem TCY

This operation burns `sTCY` and sends TCY back to the user.

### ⚙️ ExecuteMsg:
```json
{
  "liquid": {
    "unbond": {}
  }
}
```

### 📦 CLI Example:
```bash
thornode tx wasm execute thor1z7ejlk5wk2pxh9nfwjzkkdnrq4p2f5rjcpudltv0gh282dwfz6nq9g2cr0 '{"liquid": {"unbond": {}}}'   --amount 100000000x/staking-tcy   --from <wallet>   --gas auto --gas-adjustment 2   --chain-id thorchain-1   --node https://rpc.ninerealms.com
```

---

## 📊 Status Query

Returns useful data about the contract, including bonded amount and share price.

### ⚙️ QueryMsg:
```json
{
  "status": {}
}
```

### 📦 CLI Example:
```bash
thornode query wasm contract-state smart thor1z7ejlk5wk2pxh9nfwjzkkdnrq4p2f5rjcpudltv0gh282dwfz6nq9g2cr0 '{"status":{}}'   --node https://rpc.ninerealms.com/
```

### 📥 Sample Response:
```yaml
account_bond: "20585674494014"
assigned_revenue: "15913782"
liquid_bond_shares: "375244697717770"
liquid_bond_size: "375453136725787"
undistributed_revenue: "0"
```

### 📘 Field-by-field explanation

#### `account_bond`
- 🔎 **What:** Total TCY deposited in *non-compounding* mode (account staking).
- ⚠️ **Note:** If only using the official UI, this will likely be zero.

#### `assigned_revenue`
- 🔎 **What:** Amount of TCY rewards already assigned to users who deposited via `AccountMsg::Bond` (non-compounding mode).
- 📌 **Note:** This tracks rewards reserved for account bonders (non-compounding), not for `sTCY` holders.
- ⚠️ **If you're only supporting the auto-compound flow (as recommended), this will remain zero and can be ignored in the UI.

#### `liquid_bond_shares`
- 🔎 **What:** Total `sTCY` tokens minted (i.e., shares in the pool).
- 📌 **Used for:** Calculating share price.

#### `liquid_bond_size`
- 🔎 **What:** Total TCY in the pool (including compounded rewards).
- 📐 **Formula:** `share_price = liquid_bond_size / liquid_bond_shares`

#### `undistributed_revenue`
- 🔎 **What:** TCY rewards that have not yet been distributed.
- 📌 **Usually:** Close to 0 unless pending distribution.

---

## ✅ Summary

| Action          | Function Called                                      | Input               | Output         |
|----------------|-------------------------------------------------------|---------------------|----------------|
| Deposit         | `{"liquid": {"bond": {}}}`                            | TCY                 | sTCY           |
| Withdraw        | `{"liquid": {"unbond": {}}}`                          | sTCY                | TCY            |
| Check Status    | `{"status": {}}`                                      | —                   | Bond/share data|

---

If you have any questions or need help with integration, reach out to the AutoRujira team.

Happy compounding! 🧬
