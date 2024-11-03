
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

declare_id!("CCLnXJAJYFjCHLCugpBCEQKrpiSApiRM4UxkBUHJRrv4");

#[program]
pub mod token_pool {
    use super::*;

    // 初始化池子账户
    pub fn initialize_pool(ctx: Context<InitializePool>, mint_address: Pubkey) -> Result<()> {
        let pool_account = &mut ctx.accounts.pool_account;

        // 设置池子接受的代币 mint 地址
        pool_account.accepted_mint = mint_address;
        pool_account.total_deposit_amount = 0;

        Ok(())
    }

    // 接收用户转账并记录 principal 和转账金额
    pub fn deposit(ctx: Context<Deposit>, principal: String, amount: u64) -> Result<()> {
        let pool_account = &mut ctx.accounts.pool_account;

        // 检查用户转账的代币是否为指定的 SPL 代币
        if ctx.accounts.user_token_account.mint != pool_account.accepted_mint {
            return Err(ErrorCode::InvalidTokenMint.into());
        }

        // 更新池子账户信息
        pool_account.principal = principal;
        pool_account.last_deposit_amount = amount;
        pool_account.total_deposit_amount = pool_account
            .total_deposit_amount
            .checked_add(amount)
            .unwrap();

        // 执行代币转账
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.pool_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }
}

// 池子账户
#[account]
pub struct PoolAccount {
    pub principal: String,
    pub accepted_mint: Pubkey,        // 指定的 SPL 代币的 mint 地址
    pub total_deposit_amount: u64,
    pub last_deposit_amount: u64,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = user, space = 8 + 32 + 32 + 8 + 8)]
    pub pool_account: Account<'info, PoolAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub pool_account: Account<'info, PoolAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut, constraint = pool_token_account.mint == pool_account.accepted_mint)]
    pub pool_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The token mint does not match the accepted SPL token for this pool.")]
    InvalidTokenMint,
}
