// No imports needed: web3, anchor, pg and more are globally available

// TODO: Ensure the program is deployed to the correct Solana cluster and that the `declare_id!` in the program matches the program ID used in these tests. 
// Verify all required signer accounts (e.g., mint authority, staking pool authority) are properly initialized and included in transactions.
// Add the correct `TOKEN_PROGRAM_ID` for the SPL Token Program.
// Revisit and fix these issues.


const TOKEN_PROGRAM_ID = new web3.PublicKey("");  //REPLACE_WITH_TOKEN_PROGRAM_ID

describe("fresa_coin", () => {
  let mint: web3.PublicKey;
  let userTokenAccount: web3.PublicKey;
  let stakingPool: web3.PublicKey;
  let userStakeAccount: web3.PublicKey;
  const mintAuthority = web3.Keypair.generate();

  it("Initializes the token mint", async () => {
    const mintKp = web3.Keypair.generate();
    const userTokenAccountKp = web3.Keypair.generate();
    mint = mintKp.publicKey;
    userTokenAccount = userTokenAccountKp.publicKey;

    const totalSupply = new BN(1_000_000 * 10 ** 6);

    const txHash = await pg.program.methods
      .initializeToken(totalSupply)
      .accounts({
        mint: mint,
        tokenAccount: userTokenAccount,
        authority: mintAuthority.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([mintKp, userTokenAccountKp, mintAuthority])
      .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    const mintAccountInfo = await pg.connection.getParsedAccountInfo(mint);
    assert(mintAccountInfo.value !== null, "Mint account not initialized");
  });

  it("Initializes the staking pool", async () => {
    const stakingPoolKp = web3.Keypair.generate();
    stakingPool = stakingPoolKp.publicKey;

    const rewardRate = new BN(10);
    const lockDuration = new BN(7 * 24 * 60 * 60);

    const txHash = await pg.program.methods
      .initializeStakingPool(rewardRate, lockDuration)
      .accounts({
        stakingPool: stakingPool,
        authority: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([stakingPoolKp])
      .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    const stakingPoolAccount = await pg.program.account.stakingPool.fetch(stakingPool);
    assert.strictEqual(stakingPoolAccount.rewardRate.toNumber(), rewardRate.toNumber());
    assert.strictEqual(stakingPoolAccount.lockDuration.toNumber(), lockDuration.toNumber());
  });

  it("Stakes tokens", async () => {
    const userStakeAccountKp = web3.Keypair.generate();
    userStakeAccount = userStakeAccountKp.publicKey;
    const stakeAmount = new BN(500 * 10 ** 6);

    const txHash = await pg.program.methods
      .stakeTokens(stakeAmount, null) // Pass `null` for optional referrer
      .accounts({
        stakeAccount: userStakeAccount,
        userAccount: userTokenAccount,
        stakingPool: stakingPool,
        mint: mint,
        authority: pg.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        referrerAccount: null,
      })
      .signers([userStakeAccountKp])
      .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    const stakeAccount = await pg.program.account.stakeAccount.fetch(userStakeAccount);
    assert.strictEqual(stakeAccount.totalStaked.toNumber(), stakeAmount.toNumber());
  });

  it("Withdraws tokens", async () => {
    const withdrawAmount = new BN(250 * 10 ** 6);

    const txHash = await pg.program.methods
      .withdrawTokens(withdrawAmount)
      .accounts({
        stakeAccount: userStakeAccount,
        userAccount: userTokenAccount,
        stakingPool: stakingPool,
        mint: mint,
        authority: pg.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log(`Use 'solana confirm -v ${txHash}' to see the logs`);

    const stakeAccount = await pg.program.account.stakeAccount.fetch(userStakeAccount);
    assert.strictEqual(stakeAccount.totalStaked.toNumber(), withdrawAmount.toNumber());
  });
});
