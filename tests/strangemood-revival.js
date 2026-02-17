const anchor = require("@coral-xyz/anchor");
const { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } = require("@solana/web3.js");
const {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} = require("@solana/spl-token");

const CONFIRM_OPTS = { commitment: "confirmed" };

describe("strangemood-revival", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.strangemoodRevival;
  const wallet = provider.wallet;

  // Shared state across tests
  let charterMint;
  let charterVoteDeposit;
  let charterPda;
  let paymentMint;
  let treasuryPda;
  let treasuryDeposit;
  let listingMint;
  let listingPda;
  let listingPaymentDeposit;
  let listingVoteDeposit;

  it("Initializes a Charter (marketplace DAO)", async () => {
    charterMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      wallet.publicKey,
      0,
      Keypair.generate(),
      CONFIRM_OPTS
    );

    const voteDepositKp = Keypair.generate();
    charterVoteDeposit = await createAccount(
      provider.connection,
      wallet.payer,
      charterMint,
      wallet.publicKey,
      voteDepositKp,
      CONFIRM_OPTS
    );

    const [_charterPda, charterBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("charter"), charterMint.toBuffer()],
      program.programId
    );
    charterPda = _charterPda;

    await program.methods
      .initCharter(
        charterBump,
        new anchor.BN(1),
        0,
        new anchor.BN(10),
        2,
        new anchor.BN(10),
        2,
        "https://strangemood-revival.dev"
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

    const charter = await program.account.charter.fetch(charterPda);
    console.log("  Charter authority:", charter.authority.toBase58());
    console.log("  Charter URI:", charter.uri);
    assert(charter.isInitialized === true);
    assert(charter.uri === "https://strangemood-revival.dev");
  });

  it("Initializes a Charter Treasury", async () => {
    paymentMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6,
      Keypair.generate(),
      CONFIRM_OPTS
    );

    const treasuryDepositKp = Keypair.generate();
    treasuryDeposit = await createAccount(
      provider.connection,
      wallet.payer,
      paymentMint,
      wallet.publicKey,
      treasuryDepositKp,
      CONFIRM_OPTS
    );

    const [_treasuryPda, treasuryBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("treasury"),
        charterPda.toBuffer(),
        paymentMint.toBuffer(),
      ],
      program.programId
    );
    treasuryPda = _treasuryPda;

    await program.methods
      .initCharterTreasury(
        treasuryBump,
        new anchor.BN(1),
        0
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

    const treasury = await program.account.charterTreasury.fetch(treasuryPda);
    assert(treasury.isInitialized === true);
    console.log("  Treasury mint:", treasury.mint.toBase58().slice(0, 12) + "...");
  });

  it("Lists a game on the marketplace", async () => {
    const mintKeypair = Keypair.generate();
    listingMint = mintKeypair;

    const [mintAuthorityPda, mintBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint"), mintKeypair.publicKey.toBuffer()],
      program.programId
    );

    const [_listingPda, listingBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("listing"), mintKeypair.publicKey.toBuffer()],
      program.programId
    );
    listingPda = _listingPda;

    const payDepositKp = Keypair.generate();
    listingPaymentDeposit = await createAccount(
      provider.connection,
      wallet.payer,
      paymentMint,
      wallet.publicKey,
      payDepositKp,
      CONFIRM_OPTS
    );

    const voteDepositKp = Keypair.generate();
    listingVoteDeposit = await createAccount(
      provider.connection,
      wallet.payer,
      charterMint,
      wallet.publicKey,
      voteDepositKp,
      CONFIRM_OPTS
    );

    await program.methods
      .initListing(
        mintBump,
        listingBump,
        0,
        new anchor.BN(1_000_000), // price: 1 USDC
        true, // refundable
        false, // consumable
        true, // available
        "https://strangemood-revival.dev/games/super-solana-kart"
      )
      .accounts({
        listing: listingPda,
        mintAuthorityPda: mintAuthorityPda,
        mint: mintKeypair.publicKey,
        paymentDeposit: listingPaymentDeposit,
        voteDeposit: listingVoteDeposit,
        charter: charterPda,
        charterTreasury: treasuryPda,
        rent: SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        user: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([mintKeypair])
      .rpc();

    const listing = await program.account.listing.fetch(listingPda);
    console.log("  Game listed:", listing.uri);
    console.log("  Price:", listing.price.toNumber() / 1e6, "USDC");
    console.log("  Refundable:", listing.isRefundable);
    console.log("  Available:", listing.isAvailable);
    assert(listing.isInitialized === true);
    assert(listing.isAvailable === true);
    assert(listing.price.toNumber() === 1_000_000);
    assert(listing.uri === "https://strangemood-revival.dev/games/super-solana-kart");
  });

  it("Purchases a game", async () => {
    const buyerPayKp = Keypair.generate();
    const buyerPaymentAccount = await createAccount(
      provider.connection,
      wallet.payer,
      paymentMint,
      wallet.publicKey,
      buyerPayKp,
      CONFIRM_OPTS
    );

    await mintTo(
      provider.connection,
      wallet.payer,
      paymentMint,
      buyerPaymentAccount,
      wallet.publicKey,
      10_000_000, // 10 USDC
      [],
      CONFIRM_OPTS
    );

    const listingTokenKp = Keypair.generate();
    const listingTokenAccount = await createAccount(
      provider.connection,
      wallet.payer,
      listingMint.publicKey,
      wallet.publicKey,
      listingTokenKp,
      CONFIRM_OPTS
    );

    const [mintAuthorityPda, mintBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint"), listingMint.publicKey.toBuffer()],
      program.programId
    );

    const escrowKeypair = Keypair.generate();

    const [escrowAuthority, escrowBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), escrowKeypair.publicKey.toBuffer()],
      program.programId
    );

    const receiptNonce = new anchor.BN(Date.now());

    const receiptNonceBuffer = Buffer.alloc(16);
    receiptNonceBuffer.writeBigUInt64LE(BigInt(receiptNonce.toString()), 0);
    const [receiptPda, receiptBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), receiptNonceBuffer],
      program.programId
    );

    const cashier = Keypair.generate();

    await program.methods
      .purchase(
        receiptNonce,
        receiptBump,
        mintBump,
        escrowBump,
        new anchor.BN(1)
      )
      .accounts({
        purchaseTokenAccount: buyerPaymentAccount,
        listing: listingPda,
        listingPaymentDeposit: listingPaymentDeposit,
        listingPaymentDepositMint: paymentMint,
        cashier: cashier.publicKey,
        listingTokenAccount: listingTokenAccount,
        listingMint: listingMint.publicKey,
        listingMintAuthority: mintAuthorityPda,
        receipt: receiptPda,
        escrow: escrowKeypair.publicKey,
        escrowAuthority: escrowAuthority,
        user: wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([escrowKeypair])
      .rpc();

    const receipt = await program.account.receipt.fetch(receiptPda);
    console.log("  Purchase receipt created");
    console.log("  Quantity:", receipt.quantity.toNumber());
    console.log("  Price:", receipt.price.toNumber() / 1e6, "USDC");
    console.log("  Refundable:", receipt.isRefundable);
    console.log("  Cashable:", receipt.isCashable);
    assert(receipt.isInitialized === true);
    assert(receipt.quantity.toNumber() === 1);
    assert(receipt.isRefundable === true);
    assert(receipt.isCashable === false);

    const escrowAccount = await getAccount(provider.connection, escrowKeypair.publicKey);
    console.log("  Escrow balance:", Number(escrowAccount.amount) / 1e6, "USDC");
    assert(Number(escrowAccount.amount) === 1_000_000);

    const licenseAccount = await getAccount(provider.connection, listingTokenAccount);
    console.log("  License tokens:", Number(licenseAccount.amount));
    assert(Number(licenseAccount.amount) === 1);
    assert(licenseAccount.isFrozen === true);
  });

  it("Updates listing price", async () => {
    await program.methods
      .setListingPrice(new anchor.BN(2_000_000))
      .accounts({
        listing: listingPda,
        user: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const listing = await program.account.listing.fetch(listingPda);
    console.log("  New price:", listing.price.toNumber() / 1e6, "USDC");
    assert(listing.price.toNumber() === 2_000_000);
  });

  it("Updates listing availability", async () => {
    await program.methods
      .setListingAvailability(false)
      .accounts({
        listing: listingPda,
        user: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    let listing = await program.account.listing.fetch(listingPda);
    assert(listing.isAvailable === false);
    console.log("  Delisted game");

    await program.methods
      .setListingAvailability(true)
      .accounts({
        listing: listingPda,
        user: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    listing = await program.account.listing.fetch(listingPda);
    assert(listing.isAvailable === true);
    console.log("  Re-listed game");
  });

  it("Updates charter contribution rates", async () => {
    await program.methods
      .setCharterContributionRate(
        new anchor.BN(15),
        2,
        new anchor.BN(15),
        2
      )
      .accounts({
        charter: charterPda,
        user: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const charter = await program.account.charter.fetch(charterPda);
    console.log("  New payment rate:", charter.paymentContributionRateAmount.toNumber() + "%");
    assert(charter.paymentContributionRateAmount.toNumber() === 15);
  });
});

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}
