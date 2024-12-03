use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, TokenAccount, Transfer, Burn, MintTo, Token};

declare_id!("3JNQ7rDqxbrM14B8uhSWMjkvTWTPSWvCWsk9dwEPHUyn");

#[program]
pub mod fresa_coin {
    use super::*;

    // Initialize Token Mint
    pub fn initialize_token(ctx: Context<InitializeToken>, total_supply: u64) -> Result<()> {
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            total_supply,
        )?;
        Ok(())
    }

    // Initialize Staking Pool
    pub fn initialize_staking_pool(
        ctx: Context<InitializeStakingPool>,
        reward_rate: u64,
        lock_duration: i64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.staking_pool;
        pool.reward_rate = reward_rate;
        pool.lock_duration = lock_duration;
        pool.total_staked = 0;
        Ok(())
    }

    // Initialize Stake Account
    pub fn initialize_stake_account(ctx: Context<InitializeStakeAccount>) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        stake_account.total_staked = 0;
        stake_account.reward_accumulated = 0;
        stake_account.last_staked_timestamp = Clock::get()?.unix_timestamp;
        stake_account.referrer = None;
        Ok(())
    }

    // Stake Tokens
    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64, referrer: Option<Pubkey>) -> Result<()> {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_account.to_account_info(),
                    to: ctx.accounts.staking_pool.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
        )?;

        let stake_account = &mut ctx.accounts.stake_account;
        let duration = Clock::get()?.unix_timestamp - stake_account.last_staked_timestamp;

        // Update stake and rewards
        stake_account.total_staked += amount;
        stake_account.reward_accumulated += calculate_reward(amount, duration);
        stake_account.last_staked_timestamp = Clock::get()?.unix_timestamp;

        // Handle optional referrer
        if let Some(referrer_account) = &ctx.accounts.referrer_account {
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.staking_pool.to_account_info(),
                        to: referrer_account.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                ),
                amount / 20, // 5% referral bonus
            )?;
        }

        // Airdrop bonus for first-time stakers
        if stake_account.total_staked == amount {
            token::mint_to(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    MintTo {
                        mint: ctx.accounts.mint.to_account_info(),
                        to: ctx.accounts.user_account.to_account_info(),
                        authority: ctx.accounts.authority.to_account_info(),
                    },
                ),
                100 * 10_u64.pow(6), // Airdrop 100 tokens
            )?;
        }

        Ok(())
    }

    // Withdraw Tokens
    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>, amount: u64) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let duration = Clock::get()?.unix_timestamp - stake_account.last_staked_timestamp;

        let penalty = if duration < MIN_STAKE_DURATION {
            amount / 5 // 20% penalty
        } else {
            0
        };

        let net_withdraw = amount - penalty;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.staking_pool.to_account_info(),
                    to: ctx.accounts.user_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            net_withdraw,
        )?;

        // Update state after transfer
        stake_account.total_staked -= amount;

        // Burn a portion of the penalty
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.staking_pool.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            penalty / 2,
        )?;

        Ok(())
    }

    // Force Withdraw Tokens (Emergency)
    pub fn force_withdraw_tokens(ctx: Context<WithdrawTokens>, amount: u64) -> Result<()> {
        let penalty = amount / 2; // 50% penalty for emergency withdrawal
        let net_withdraw = amount - penalty;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.staking_pool.to_account_info(),
                    to: ctx.accounts.user_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            net_withdraw,
        )?;

        // Burn a portion of the penalty
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.staking_pool.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            penalty,
        )?;

        Ok(())
    }

    // Submit Governance Proposal
    pub fn submit_proposal(ctx: Context<SubmitProposal>, description: String) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        proposal.description = description;
        proposal.votes_for = 0;
        proposal.votes_against = 0;
        proposal.is_approved = false;
        Ok(())
    }

    // Vote on Governance Proposal with Token Weight
    pub fn vote(ctx: Context<Vote>, vote_for: bool) -> Result<()> {
        let stake_account = &ctx.accounts.stake_account;
        let weight = stake_account.total_staked; // Use staked tokens as voting weight

        let proposal = &mut ctx.accounts.proposal;
        if vote_for {
            proposal.votes_for += weight;
        } else {
            proposal.votes_against += weight;
        }
        proposal.is_approved = proposal.votes_for > proposal.votes_against;
        Ok(())
    }

    // Draw Lottery Winner
    pub fn draw_lottery(ctx: Context<DrawLottery>) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;

        let random_index = Clock::get()?.unix_timestamp as usize % lottery.entries.len();
        let winner = lottery.entries[random_index];

        // Transfer prize pool to the winner
        let prize_pool = lottery.prize_pool;
        lottery.prize_pool = 0;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.staking_pool.to_account_info(),
                    to: ctx.accounts.winner_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            prize_pool,
        )?;

        Ok(())
    }
}

// Helper Function to Calculate Rewards
fn calculate_reward(amount: u64, duration: i64) -> u64 {
    let base_rate = if amount > 10_000 * 10_u64.pow(6) {
        15 // 15% for staking > 10,000 Fresa Coins
    } else if amount > 1_000 * 10_u64.pow(6) {
        12 // 12% for staking > 1,000 Fresa Coins
    } else {
        10 // 10% for everyone else
    };

    let multiplier = if duration > 30 * 24 * 60 * 60 {
        2 // Double rewards for staking > 30 days
    } else {
        1
    };

    (amount * base_rate / 100) * multiplier
}

// Constants
const MIN_STAKE_DURATION: i64 = 7 * 24 * 60 * 60; // 7 days

// Account Structures
#[derive(Accounts)]
pub struct InitializeToken<'info> {
    #[account(init, payer = authority, mint::decimals = 6, mint::authority = authority)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, token::mint = mint, token::authority = authority)]
    pub token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeStakingPool<'info> {
    #[account(init, payer = authority, space = 8 + 48)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeStakeAccount<'info> {
    #[account(init, payer = authority, space = 8 + 128)]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    pub user_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub staking_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub referrer_account: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
    #[account(mut)]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(mut)]
    pub user_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub staking_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DrawLottery<'info> {
    #[account(mut)]
    pub lottery: Account<'info, Lottery>,
    #[account(mut)]
    pub staking_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub winner_account: Account<'info, TokenAccount>, // Winner account for prize transfer
    pub token_program: Program<'info, Token>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SubmitProposal<'info> {
    #[account(init, payer = authority, space = 8 + 128)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Vote<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub stake_account: Account<'info, StakeAccount>,
    pub authority: Signer<'info>,
}

#[account]
pub struct StakeAccount {
    pub total_staked: u64,
    pub reward_accumulated: u64,
    pub last_staked_timestamp: i64,
    pub referrer: Option<Pubkey>,
}

#[account]
pub struct StakingPool {
    pub reward_rate: u64,
    pub lock_duration: i64,
    pub total_staked: u64,
}

#[account]
pub struct Proposal {
    pub description: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub is_approved: bool,
}

#[account]
pub struct Lottery {
    pub entries: Vec<Pubkey>,
    pub prize_pool: u64,
    pub last_draw: i64,
}
