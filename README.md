# fresa_coin
# Overview

Fresa Coin is a blockchain-based program built using the Anchor framework on the Solana blockchain. The program implements several functionalities to support a decentralized ecosystem, including token minting, staking rewards, governance, lottery, and token burning. This project was developed in the Solana Playground IDE.(https://beta.solpg.io/)

## Features

### Token Minting
**Function**: `initialize_token`
- Mints a new token with a specified total supply.
- Initializes a token account for the mint.

### Staking
#### Function: `initialize_staking_pool`
- Sets up a staking pool with configurable reward rates and lock durations.

#### Function: `initialize_stake_account`
- Initializes an account for users to participate in staking.

#### Function: `stake_tokens`
- Allows users to stake tokens in the staking pool.
- Updates rewards based on staking duration.
- Supports optional referral bonuses.
- Includes an airdrop bonus for first-time stakers.

#### Function: `withdraw_tokens`
- Allows users to withdraw staked tokens.
- Includes penalties for early withdrawals.
- Burns a portion of the penalties as a deflationary mechanism.

#### Function: `force_withdraw_tokens`
- Emergency withdrawal with a 50% penalty.

### Governance
#### Function: `submit_proposal`
- Allows users to submit governance proposals.

#### Function: `vote`
- Users vote on proposals using staked tokens as voting weight.

### Lottery
#### Function: `draw_lottery`
- Conducts a lottery for participants and distributes the prize pool to the winner.

### Token Burning
- Burn mechanisms are incorporated into staking penalties and other functionalities to reduce token supply.

## Account Structures

### `InitializeToken`
**Accounts:**
- `mint`: The token mint account.
- `authority`: The authority for the mint.
- `token_account`: The associated token account.
- `system_program`: System program for account initialization.
- `token_program`: SPL Token program.
- `rent`: Rent system variable.

### `InitializeStakingPool`
**Accounts:**
- `staking_pool`: The staking pool account.
- `authority`: The account initializing the pool.
- `system_program`: System program for account initialization.

### `InitializeStakeAccount`
**Accounts:**
- `stake_account`: The user's staking account.
- `authority`: The account initializing the staking account.
- `system_program`: System program for account initialization.

### `StakeTokens`
**Accounts:**
- `stake_account`: The user's staking account.
- `user_account`: The user's token account.
- `staking_pool`: The staking pool account.
- `referrer_account`: (Optional) Referrer's account for bonus rewards.
- `mint`: The token mint account.
- `authority`: The staking authority.
- `token_program`: SPL Token program.

### `WithdrawTokens`
**Accounts:**
- `stake_account`: The user's staking account.
- `user_account`: The user's token account.
- `staking_pool`: The staking pool account.
- `mint`: The token mint account.
- `authority`: The withdrawal authority.
- `token_program`: SPL Token program.

### `SubmitProposal`
**Accounts:**
- `proposal`: The proposal account.
- `authority`: The account submitting the proposal.
- `system_program`: System program for account initialization.

### `Vote`
**Accounts:**
- `proposal`: The proposal account being voted on.
- `stake_account`: The user's staking account.
- `authority`: The voting authority.

### `DrawLottery`
**Accounts:**
- `lottery`: The lottery account.
- `staking_pool`: The staking pool account.
- `winner_account`: The winner’s token account.
- `token_program`: SPL Token program.
- `authority`: The lottery authority.

## Constants
- **MIN_STAKE_DURATION**: Minimum duration for staking to avoid penalties (7 days).

## Helper Functions

### `calculate_reward`
- Computes rewards based on staking amount and duration.
- Includes bonus multipliers for longer staking periods.
