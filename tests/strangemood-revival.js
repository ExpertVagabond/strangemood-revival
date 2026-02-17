const anchor = require("@coral-xyz/anchor");
const { PublicKey, Keypair, SystemProgram } = require("@solana/web3.js");
const {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} = require("@solana/spl-token");

describe("strangemood-revival", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.strangemoodRevival;
  const wallet = provider.wallet;

  let charterMint;
  let charterVoteDeposit;

  it("Initializes a Charter (marketplace DAO)", async () => {
    // Create the governance token mint for the charter
    charterMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey, // mint authority (will transfer to PDA)
      wallet.publicKey, // freeze authority
      0 // decimals
    );

    // Create vote deposit token account for the charter
    charterVoteDeposit = await createAccount(
      provider.connection,
      wallet.payer,
      charterMint,
      wallet.publicKey
    );

    // Derive the charter PDA
    const [charterPda, charterBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("charter"), charterMint.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .initCharter(
        charterBump,
        new anchor.BN(1), // expansion_rate_amount
        0, // expansion_rate_decimals (1.0)
        new anchor.BN(10), // sol_contribution_rate_amount
        2, // sol_contribution_rate_decimals (0.10 = 10%)
        new anchor.BN(10), // vote_contribution_rate_amount
        2, // vote_contribution_rate_decimals (0.10 = 10%)
        "https://strangemood-revival.dev" // uri
      )
      .accounts({
        charter: charterPda,
        mint: charterMint,
        authority: wallet.publicKey,
        voteDeposit: charterVoteDeposit,
        user: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("  Charter initialized:", tx.slice(0, 20) + "...");

    // Verify charter state
    const charter = await program.account.charter.fetch(charterPda);
    console.log("  Charter authority:", charter.authority.toBase58());
    console.log("  Charter URI:", charter.uri);
    console.log("  Payment contribution: 10%");
    console.log("  Vote contribution: 10%");

    assert(charter.isInitialized === true);
    assert(charter.uri === "https://strangemood-revival.dev");
  });

  it("Initializes a Charter Treasury", async () => {
    // Create a payment token mint (simulating wrapped SOL or USDC)
    const paymentMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6 // 6 decimals like USDC
    );

    // Create deposit account for treasury
    const treasuryDeposit = await createAccount(
      provider.connection,
      wallet.payer,
      paymentMint,
      wallet.publicKey
    );

    // Derive charter PDA
    const [charterPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("charter"), charterMint.toBuffer()],
      program.programId
    );

    // Derive treasury PDA
    const [treasuryPda, treasuryBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("treasury"),
        charterPda.toBuffer(),
        paymentMint.toBuffer(),
      ],
      program.programId
    );

    const tx = await program.methods
      .initCharterTreasury(
        treasuryBump,
        new anchor.BN(1), // expansion_scalar_amount
        0 // expansion_scalar_decimals (1.0x)
      )
      .accounts({
        treasury: treasuryPda,
        charter: charterPda,
        deposit: treasuryDeposit,
        mint: paymentMint,
        authority: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("  Treasury initialized:", tx.slice(0, 20) + "...");

    const treasury = await program.account.charterTreasury.fetch(treasuryPda);
    console.log("  Treasury charter:", treasury.charter.toBase58().slice(0, 12) + "...");
    assert(treasury.isInitialized === true);
  });
});

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}
