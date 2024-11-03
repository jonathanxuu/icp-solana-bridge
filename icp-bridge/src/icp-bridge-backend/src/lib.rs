ic_cdk::export_candid!();
use candid::{CandidType, Principal, Nat};
use ic_cdk::api::call::call;
use ic_cdk::caller;
use icrc_ledger_types::icrc1::account::{Account, Subaccount};
use icrc_ledger_types::icrc1::transfer::Memo;
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};
use serde::{Deserialize};
use std::collections::HashMap;
use std::cell::RefCell;
use std::fmt::format;
mod threshold_schnorr;

#[derive(CandidType, Deserialize, Default, Clone)]
struct UserBalance {
    amount: u64,
}

thread_local! {
    static USER_BALANCES: RefCell<HashMap<Principal, UserBalance>> = RefCell::new(HashMap::new());
    static POOL_BALANCE: RefCell<u64> = RefCell::new(0);
}

fn convert_option_memo(option: Option<Vec<u8>>) -> Option<Memo> {
    option.map(|vec| Memo::from(vec)) // 使用 Memo::from 来转换 Vec<u8> 为 Memo
}

fn concat_u64_and_string(value: u64, text: String) -> String {
    format!("{}_{}", value, text)
}

#[ic_cdk::update]
async fn deposit_to_pool(from_subaccount: Option<Subaccount>, amount: u64, spender_subaccount: Option<Subaccount>, memo: Option<Vec<u8>>) -> Result<u64, String> {
    let icrc2_canister_id = Principal::from_text("avqkn-guaaa-aaaaa-qaaea-cai").unwrap();

    // 设置 `from` 和 `to` 账户
    let from_account = Account {
        owner: ic_cdk::caller(),
        subaccount: from_subaccount,
    };
    let pool_account = Account {
        owner: ic_cdk::id(),
        subaccount: None,
    };

    // 构建 `TransferFromArgs`
    let transfer_args = TransferFromArgs {
        spender_subaccount,
        from: from_account,
        to: pool_account,
        amount: amount.into(),
        fee: None,
        memo: convert_option_memo(memo),
        created_at_time: None,
    };

    // 调用 `icrc2_transfer_from` 方法从用户账户转账到池子账户
    let (result, ):(Result<Nat, TransferFromError>,) = call(icrc2_canister_id, "icrc2_transfer_from", (transfer_args,))
        .await
        .map_err(|err| format!("Transfer failed: {:?}", err))?;

    print!("Transfer result: {:?}", result);
    // 更新用户余额和池子余额
    let caller = ic_cdk::caller();
    USER_BALANCES.with(|balances| {
        let mut balances = balances.borrow_mut();
        let user_balance = balances.entry(caller).or_insert_with(UserBalance::default);
        user_balance.amount += amount;
    });

    POOL_BALANCE.with(|balance| *balance.borrow_mut() += amount);

    // todo, for now just u64 size
    Ok(result.unwrap().0.to_u64_digits()[0])
}


#[ic_cdk::update]
async fn bridge_to_solana(amount: u64, solana_address: String) -> Result<String, String> {
    let caller = caller();
    
    // 检查用户余额
    let user_balance = USER_BALANCES.with(|balances| {
        let balances = balances.borrow();
        balances.get(&caller).cloned().unwrap_or_default()
    });

    // 确保用户有足够的余额
    if user_balance.amount < amount {
        return Err("Insufficient balance".to_string());
    }

    // the verification work for solana_address, leaves to the frontend
    // if !is_valid_solana_address(address) {
    //     return Err("Invalid Solana Address".to_string());
    // }

    // 更新用户余额和池子余额
    USER_BALANCES.with(|balances| {
        let mut balances = balances.borrow_mut();
        let user_balance = balances.entry(caller).or_insert_with(UserBalance::default);
        user_balance.amount -= amount; // 从用户余额中扣除
    });

    POOL_BALANCE.with(|pool_balance| {
        let mut balance = pool_balance.borrow_mut();
        *balance -= amount; // 从池子余额中扣除
    });

    let signature = threshold_schnorr::schnorr_sign(concat_u64_and_string(amount, solana_address))
        .await
        .map_err(|e| format!("Sign Failed: {}", e))?;

    return Ok(signature.signature_hex);
}



#[ic_cdk::query]
fn get_user_balance(user: Principal) -> u64 {
    USER_BALANCES.with(|balances| {
        balances
            .borrow()
            .get(&user)
            .map(|balance| balance.amount)
            .unwrap_or(0)
    })
}

#[ic_cdk::query]
fn get_pool_balance() -> u64 {
    POOL_BALANCE.with(|balance| *balance.borrow())
}


#[ic_cdk::query]
fn get_canister_id() -> String {
    // 获取当前 Canister 的 ID
    let canister_id = ic_cdk::id();
    // 将 Canister ID 转换为字符串并返回
    canister_id.to_string()
}
