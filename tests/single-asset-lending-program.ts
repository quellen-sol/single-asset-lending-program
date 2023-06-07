import { AnchorProvider, workspace, Program, getProvider, setProvider, IdlAccounts } from "@coral-xyz/anchor";
import { SingleAssetLendingProgram } from "../target/types/single_asset_lending_program";
import { Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, SYSVAR_RENT_PUBKEY, SystemProgram } from "@solana/web3.js";
import { AccountLayout, TOKEN_PROGRAM_ID, createAssociatedTokenAccount, createMint, mintTo } from "@solana/spl-token";
import { assert, expect } from "chai";
import { BN } from "bn.js";

setProvider(AnchorProvider.env());

const program = workspace.SingleAssetLendingProgram as Program<SingleAssetLendingProgram>;

async function airdrop(connection: Connection, pubkey: PublicKey, amount) {
  const sig = await connection.requestAirdrop(pubkey, amount);
  await connection.confirmTransaction(sig);
}

async function setup() {
  const { connection } = getProvider();

  const alice = Keypair.generate();
  const bob = Keypair.generate();
  const malicious = Keypair.generate();

  // Seed them with SOL
  const initial_lamports = 1 * LAMPORTS_PER_SOL;
  await airdrop(connection, alice.publicKey, initial_lamports);
  await airdrop(connection, bob.publicKey, initial_lamports);
  await airdrop(connection, malicious.publicKey, initial_lamports);

  return {
    alice,
    bob,
    malicious,
  };
}

const VAULT_SEED = "vault";
const VAULT_STATE_SEED = "state";
const VAULT_REWARDS_SEED = "rewards";

function deriveVaultAddress(mint: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(VAULT_SEED),
      mint.toBuffer(),
    ],
    program.programId,
  );
}

function deriveVaultStateAddress(vault: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(VAULT_STATE_SEED),
      vault.toBuffer(),
    ],
    program.programId,
  )[0];
}

function deriveVaultRewardsAddress(vault: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(VAULT_REWARDS_SEED),
      vault.toBuffer(),
    ],
    program.programId,
  )[0];
}

function deriveUserVaultAddress(vault: PublicKey, user: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      vault.toBuffer(),
      user.toBuffer(),
    ],
    program.programId,
  )[0];
}

function calcRewardRatio(vault: IdlAccounts<SingleAssetLendingProgram>["vaultState"]) {
  return vault.totalDeposits.toNumber() / vault.rewardFactor;
}

describe("single-asset-lending-program", () => {
  let alice: Keypair;
  let bob: Keypair;
  let malicious: Keypair;
  let vaultMint: PublicKey;
  let aliceATA: PublicKey;
  let bobATA: PublicKey;
  let maliciousATA: PublicKey;

  let vaultAddress: PublicKey;
  let vaultBump: number;
  let vaultStateAddress: PublicKey;
  let vaultRewardsAddress: PublicKey;

  let aliceUserStateAddress: PublicKey;
  let bobUserStateAddress: PublicKey;
  let maliciousUserStateAddress: PublicKey;

  const { connection } = getProvider();

  before(async () => {
    const result = await setup();
    alice = result.alice;
    bob = result.bob;
    malicious = result.malicious;

    // Create a new mint
    vaultMint = await createMint(connection, alice, alice.publicKey, alice.publicKey, 9, undefined, undefined, TOKEN_PROGRAM_ID);

    // Mint to all parties
    aliceATA = await createAssociatedTokenAccount(connection, alice, vaultMint, alice.publicKey);
    bobATA = await createAssociatedTokenAccount(connection, bob, vaultMint, bob.publicKey);
    maliciousATA = await createAssociatedTokenAccount(connection, malicious, vaultMint, malicious.publicKey);

    await mintTo(connection, alice, vaultMint, aliceATA, alice, 1000 * LAMPORTS_PER_SOL);
    await mintTo(connection, bob, vaultMint, bobATA, alice, 1000 * LAMPORTS_PER_SOL);
    await mintTo(connection, malicious, vaultMint, maliciousATA, alice, 1000 * LAMPORTS_PER_SOL);

    const [vAddress, bump] = deriveVaultAddress(vaultMint);
    vaultAddress = vAddress;
    vaultBump = vaultBump;
    vaultStateAddress = deriveVaultStateAddress(vaultAddress);
    vaultRewardsAddress = deriveVaultRewardsAddress(vaultAddress);

    aliceUserStateAddress = deriveUserVaultAddress(vaultAddress, alice.publicKey);
    bobUserStateAddress = deriveUserVaultAddress(vaultAddress, bob.publicKey);
    maliciousUserStateAddress = deriveUserVaultAddress(vaultAddress, malicious.publicKey);
  });

  it("Setup worked correctly", async () => {
    const b = await connection.getTokenAccountBalance(aliceATA);
    const supply = await connection.getTokenSupply(vaultMint);
    expect(supply.value.uiAmount).to.equal(3000);
    expect(b.value.uiAmount).to.equal(1000);
  });

  it("Alice initializes a vault", async () => {
    await program.methods.createVault(0.02, 0.8).accounts({
      payer: alice.publicKey,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      vaultAccount: vaultAddress,
      vaultStateAccount: vaultStateAddress,
      vaultRewardsAccount: vaultRewardsAddress,
      vaultMint,
    }).signers([alice]).rpc();

    // Check that the vault was created
    const vaultAccount = await connection.getAccountInfo(vaultAddress);
    const decoded = AccountLayout.decode(vaultAccount.data);
    console.log(decoded);
    expect(decoded.owner.toBase58()).to.equal(vaultAddress.toBase58());    
  });

  it("Alice deposits 100 token into vault", async () => {
    await program.methods.deposit(new BN(100 * LAMPORTS_PER_SOL)).accounts({
      payer: alice.publicKey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      userStateAccount: aliceUserStateAddress,
      userTokenAccount: aliceATA,
      vaultAccount: vaultAddress,
      vaultMint,
      vaultRewardsAccount: vaultRewardsAddress,
      vaultStateAccount: vaultStateAddress,
    }).signers([alice]).rpc();

    // Check that the vault actually received the tokens
    const vault = await connection.getTokenAccountBalance(vaultAddress);
    expect(vault.value.uiAmount).to.equal(100);

    // Check reward ratio
    // const vault_state = await program.account.vaultState.fetch(vaultStateAddress);
    // const reward_ratio = calcRewardRatio(vault_state);
    // console.log("Reward ratio", reward_ratio);
    // expect(vault_state.rewardFactor).to.equal(0.8);
  });

  it("Bob deposits 50 token into vault", async () => {
    await program.methods.deposit(new BN(50 * LAMPORTS_PER_SOL)).accounts({
      payer: bob.publicKey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      userStateAccount: bobUserStateAddress,
      userTokenAccount: bobATA,
      vaultAccount: vaultAddress,
      vaultMint,
      vaultRewardsAccount: vaultRewardsAddress,
      vaultStateAccount: vaultStateAddress,
    }).signers([bob]).rpc();
  });

  it("Alice tries to borrow her max", async () => {
    await program.methods.borrow(vaultBump, new BN(80 * LAMPORTS_PER_SOL)).accounts({
      payer: alice.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
      userStateAccount: aliceUserStateAddress,
      userTokenAcccount: aliceATA,
      vaultAccount: vaultAddress,
      vaultMint,
      vaultStateAccount: vaultStateAddress,
    }).signers([alice]).rpc({
      skipPreflight: true,
    });
    // .then(() => assert(false), () => assert(true));
  });
});
