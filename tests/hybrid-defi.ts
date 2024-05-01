import * as anchor from "@coral-xyz/anchor";
import { readFileSync } from "fs";
import path from "path";
import * as spl from "@solana/spl-token";
import { SystemProgram } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { utf8 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import {
  Signer,
  createSignerFromKeypair,
  generateSigner,
  percentAmount,
  publicKey,
  signerIdentity,
  sol,
  some,
} from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  MPL_TOKEN_METADATA_PROGRAM_ID,
  createNft,
  createProgrammableNft,
  findMasterEditionPda,
  findMetadataPda,
  findTokenRecordPda,
  mplTokenMetadata,
  verifyCollectionV1,
} from "@metaplex-foundation/mpl-token-metadata";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddressSync,
  mintTo,
} from "@solana/spl-token";
import { SPL_SYSTEM_PROGRAM_ID, SPL_TOKEN_PROGRAM_ID, findAssociatedTokenPda } from "@metaplex-foundation/mpl-toolbox";
import { assert } from "chai";
import { HybridDefi } from "../target/types/hybrid_defi";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("hybrid-defi", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.HybridDefi as Program<HybridDefi>;

  const umi = createUmi(anchor.AnchorProvider.env().connection.rpcEndpoint).use(
    mplTokenMetadata()
  );

  const keyFileContents = JSON.parse(
    readFileSync(
      path.join(process.env.HOME, ".config/solana/id.json")
    ).toString()
  );

   const signer = umi.eddsa.createKeypairFromSecretKey(
    new Uint8Array(keyFileContents)
  );

  let feeWallets = ["EYNsuoUh4pRCpuNqj5cH8zUDXST4o8YYqRg6vraG7Br7", "3nHNJd8mjZFTVkA2dPTSCnWjzJU3XvC5nGSrDMWNKpQb", "ghosnnrbJRNUueziNL579JZCqvcLpdHSMXU2zn9uGJS"];

  const signerKp = anchor.web3.Keypair.fromSecretKey(new Uint8Array(keyFileContents)) as anchor.web3.Keypair;

  umi.use(signerIdentity(createSignerFromKeypair(umi, signer)));

  const tokenProgram = new anchor.web3.PublicKey(SPL_TOKEN_PROGRAM_ID);
  const metadataProgram = new anchor.web3.PublicKey(
    MPL_TOKEN_METADATA_PROGRAM_ID
  );
  const associatedTokenProgram = new anchor.web3.PublicKey(
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  const systemProgram = new anchor.web3.PublicKey(SPL_SYSTEM_PROGRAM_ID);
  const sysvarInstructions = new anchor.web3.PublicKey(
    anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY
  );

  // Collection
  const collectionMint = generateSigner(umi);
  const collectionMintPubkey = new anchor.web3.PublicKey(
    collectionMint.publicKey
  );

  // NFT of the collection that must be owned by the Signer
  const nftMint = generateSigner(umi);
  const nftMintPubkey = new anchor.web3.PublicKey(nftMint.publicKey);

  const nftToken = findAssociatedTokenPda(umi, {
    mint: nftMint.publicKey,
    owner: umi.identity.publicKey,
  });
  const nftTokenPubkey = new anchor.web3.PublicKey(publicKey(nftToken));

  const nftMetadata = findMetadataPda(umi, { mint: nftMint.publicKey });
  const nftMetadataPubkey = new anchor.web3.PublicKey(publicKey(nftMetadata));

  const nftEdition = findMasterEditionPda(umi, { mint: nftMint.publicKey });
  const nftEditionPubkey = new anchor.web3.PublicKey(publicKey(nftEdition));

  const [sponsorPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("hybrid_sponsor"), program.provider.publicKey.toBuffer(), collectionMintPubkey.toBuffer()],
        program.programId
  );

  const [nftAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("nft_authority"), sponsorPDA.toBuffer()],
    program.programId
  );

  const nftCustody = getAssociatedTokenAddressSync(
    nftMintPubkey,
    nftAuthorityPda,
    true,
    tokenProgram,
    associatedTokenProgram
  );

  const sourceTokenRecord = findTokenRecordPda(umi, {
    mint: nftMint.publicKey,
    token: publicKey(nftToken),
  });
  const sourceTokenRecordPubkey = new anchor.web3.PublicKey(
    publicKey(sourceTokenRecord)
  );

  const destinationTokenRecord = findTokenRecordPda(umi, {
    mint: nftMint.publicKey,
    token: publicKey(nftCustody),
  });

  const destinationTokenRecordPubkey = new anchor.web3.PublicKey(
    publicKey(destinationTokenRecord)
  );

  const swapFee = new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL);

  const payerKp = anchor.web3.Keypair.generate();

  let tokenMint;

  before(async () => {
    console.log("✨ Airdropping...");
    await provider.connection.requestAirdrop(payerKp.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(signerKp.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    
    await provider.connection.requestAirdrop(new anchor.web3.PublicKey(feeWallets[0]), 5 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(new anchor.web3.PublicKey(feeWallets[1]), 5 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(new anchor.web3.PublicKey(feeWallets[2]), 5 * anchor.web3.LAMPORTS_PER_SOL);

    await umi.rpc.airdrop(umi.payer.publicKey, sol(10));

    console.log("✨ Creating collection NFT...");
    await createNft(umi, {
      mint: collectionMint,
      name: "Test Collection",
      uri: "https://example.xyz",
      sellerFeeBasisPoints: percentAmount(5),
      isCollection: true,
    }).sendAndConfirm(umi);

    console.log("✨ Creating a pNFT...");
    await createProgrammableNft(umi, {
      mint: nftMint,
      tokenOwner: umi.identity.publicKey,
      name: "Test pNFT",
      uri: "https://example.xyz",
      symbol: "PCWN",
      sellerFeeBasisPoints: percentAmount(5),
      collection: some({ 
        key: collectionMint.publicKey, 
        verified: false 
      }),
    }).sendAndConfirm(umi);

    console.log("✨ Verifying...");
    await verifyCollectionV1(umi, {
      metadata: nftMetadata,
      collectionMint: collectionMint.publicKey,
      authority: umi.payer,
    }).sendAndConfirm(umi);

    console.log("✨ Creating token mint...");
    tokenMint = await createMint(
      anchor.AnchorProvider.env().connection,
      signerKp,
      signerKp.publicKey,
      signerKp.publicKey,
      6
    );

    console.log("---- 🚀 Preflight initialization completed! -----");
  });

  it("Init Sponsor", async () => {
    console.log("✨ Creating pool...");
    const tx = await program.methods.initializeSponsorPool(
      [1, 2, 3],
      new anchor.BN(1)
    ).accounts({
      hybridVault: sponsorPDA,
      tokenMint: tokenMint,
      collectionMint: collectionMintPubkey,
      nftAuthority: nftAuthorityPda,
      payer: signerKp.publicKey,
      systemProgram: systemProgram,
    }).rpc();
    console.log("✅ Pool Created! Signature:", tx.toString());

    const account = await program.account.sponsor.fetch(sponsorPDA);

    assert.exists(account);
  });

  it("Deposit Initial Tokens", async () => {
    console.log("✨ Creating sponsor token account...");
    let sponsorTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
      provider.connection,
      signerKp,
      tokenMint,
      sponsorPDA,
      true
    );

    console.log("✨ Creating payer token account...");
    let payerTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
      provider.connection,
      signerKp,
      tokenMint,
      signerKp.publicKey,
      false,
    );

    console.log("✨ Minting 1000 tokens...");
    await mintTo(
      provider.connection,
      signerKp,
      tokenMint,
      payerTokenAccount.address,
      signerKp.publicKey,
      1000
    );

    console.log("✨ Depositing 1000 tokens...");
    const deposit = await program.methods.depositTokens(
      new anchor.BN(1000)
    ).accounts({
      hybridVault: sponsorPDA,
      tokenMint: tokenMint,
      collectionMint: collectionMintPubkey,
      payerTokenAccount: payerTokenAccount.address,
      sponsorTokenAccount: sponsorTokenAccount.address,
      payer: signerKp.publicKey,
      systemProgram: systemProgram,
      tokenProgram: tokenProgram,
      associatedTokenProgram: associatedTokenProgram
    }).rpc();
    console.log("✅ Deposited! Signature:", deposit.toString());

    console.log("✨ Refetching sponsor token account...");
    let refSponsorATA = await spl.getOrCreateAssociatedTokenAccount(
      provider.connection,
      signerKp,
      tokenMint,
      sponsorPDA,
      true
    );

    assert.equal(refSponsorATA.amount, BigInt(1000));
  });

  it("Swap NFT to Token", async () => {
    console.log("✨ Fetching sponsor token account...");
    let sponsorTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
      provider.connection,
      signerKp,
      tokenMint,
      sponsorPDA,
      true
    );

    console.log("✨ Fetching payer token account...");
    let payerTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
      provider.connection,
      signerKp,
      tokenMint,
      signerKp.publicKey,
      false,
    );

    /*
    console.log(`---ACCOUNTS---\n
    sponsor: ${sponsorPDA.toString()} : ${await provider.connection.getBalance(sponsorPDA)}\n
    tokenMint: ${tokenMint} : ${await provider.connection.getBalance(tokenMint)}\n
    nftToken: ${nftTokenPubkey.toString()} : ${await provider.connection.getBalance(nftTokenPubkey)}\n
    nftMint: ${nftMintPubkey.toString()} : ${await provider.connection.getBalance(nftMintPubkey)}\n
    nftMetadata: ${nftMetadataPubkey.toString} : ${await provider.connection.getBalance(nftMetadataPubkey)}\n
    nftAuthority: ${nftAuthorityPda.toString()} : ${await provider.connection.getBalance(nftAuthorityPda)}\n
    nftCustody: ${nftCustody.toString()} : Doesnt exist yet\n
    nftEdition: ${nftEditionPubkey.toString()} : ${await provider.connection.getBalance(nftEditionPubkey)}\n
    payer: ${signerKp.publicKey.toString()} : ${await provider.connection.getBalance(signerKp.publicKey)}\n
    sourceTokenRecord: ${sourceTokenRecordPubkey.toString()} : ${await provider.connection.getBalance(sourceTokenRecordPubkey)}\n
    destinationTokenRecord: ${destinationTokenRecordPubkey.toString()} : ${await provider.connection.getBalance(destinationTokenRecordPubkey)}\n
    payerTokenAccount: ${payerTokenAccount.address} : ${await provider.connection.getBalance(payerTokenAccount.address)}\n
    sponsorTokenAccount: ${sponsorTokenAccount.address} : ${await provider.connection.getBalance(sponsorTokenAccount.address)}\n
    feeWallet: ${feeWallets[0]} : Wallet\n
    feeWalletTwo: ${feeWallets[1]} : Wallet\n
    feeWalletThree: ${feeWallets[2]} : Wallet\n
    metadataProgram: metadataProgram,
    systemProgram: systemProgram,
    tokenProgram: tokenProgram,
    associatedTokenProgram: associatedTokenProgram,
    sysvarInstructions: sysvarInstructions,
    `);
    */

    console.log("✨ Swapping...");
    const swapNFT = await program.methods.swapNftToToken().accounts({
      sponsor: sponsorPDA,
      tokenMint: tokenMint,
      nftToken: nftTokenPubkey,
      nftMint: nftMintPubkey,
      nftMetadata: nftMetadataPubkey,
      nftAuthority: nftAuthorityPda,
      nftCustody: nftCustody,
      nftEdition: nftEditionPubkey,
      payer: signerKp.publicKey,
      sourceTokenRecord: sourceTokenRecordPubkey,
      destinationTokenRecord: destinationTokenRecordPubkey,
      payerTokenAccount: payerTokenAccount.address,
      sponsorTokenAccount: sponsorTokenAccount.address,
      feeWallet: new anchor.web3.PublicKey(feeWallets[0]),
      feeWalletTwo: new anchor.web3.PublicKey(feeWallets[1]),
      feeWalletThree: new anchor.web3.PublicKey(feeWallets[2]),
      metadataProgram: metadataProgram,
      systemProgram: systemProgram,
      tokenProgram: tokenProgram,
      associatedTokenProgram: associatedTokenProgram,
      sysvarInstructions: sysvarInstructions,
    })
    .preInstructions([
      anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
        units: 400_000,
      }),
    ])
    .rpc({ skipPreflight: true });
    console.log("✅ NFT swapped to token! Signature:", swapNFT);

    const account = await program.account.sponsor.fetch(sponsorPDA);
    const nftCustodyBalance =
        await provider.connection.getTokenAccountBalance(nftCustody);

    assert.equal(nftCustodyBalance.value.uiAmount, 1)
  });

  it("Swap 2nd NFT to Token", async () => {
    console.log("✨ Fetching sponsor token account...");
    let sponsorTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
      provider.connection,
      signerKp,
      tokenMint,
      sponsorPDA,
      true
    );

    console.log("✨ Fetching payer token account...");
    let payerTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
      provider.connection,
      signerKp,
      tokenMint,
      signerKp.publicKey,
      false,
    );

    const swapNftMint = generateSigner(umi);

    const swapNftToken = findAssociatedTokenPda(umi, {
      mint: swapNftMint.publicKey,
      owner: umi.identity.publicKey,
    });

    const swapNftMetadata = findMetadataPda(umi, {
      mint: swapNftMint.publicKey,
    });

    const swapNftEdition = findMasterEditionPda(umi, {
      mint: swapNftMint.publicKey,
    });

    const sourceTokenRecord2 = findTokenRecordPda(umi, {
      mint: swapNftMint.publicKey,
      token: publicKey(swapNftToken),
    });
    const sourceTokenRecordPubkey2 = new anchor.web3.PublicKey(
      publicKey(sourceTokenRecord2)
    );

    const nftCustody2 = getAssociatedTokenAddressSync(
      new anchor.web3.PublicKey(publicKey(swapNftMint)),
      nftAuthorityPda,
      true,
      tokenProgram,
      associatedTokenProgram
    );

    const destinationTokenRecord2 = findTokenRecordPda(umi, {
      mint: swapNftMint.publicKey,
      token: publicKey(nftCustody2),
    });
  
    const destinationTokenRecordPubkey2 = new anchor.web3.PublicKey(
      publicKey(destinationTokenRecord2)
    );

    console.log("✨ Creating another pNFT...");
    await createProgrammableNft(umi, {
      mint: swapNftMint,
      tokenOwner: umi.identity.publicKey,
      name: "Test pNFT #2",
      uri: "https://example.xyz",
      symbol: "PCWN",
      sellerFeeBasisPoints: percentAmount(5),
      collection: some({ 
        key: collectionMint.publicKey, 
        verified: false 
      }),
    }).sendAndConfirm(umi);

    console.log("✨ Verifying collection on swap pNFT...");
    await verifyCollectionV1(umi, {
      metadata: swapNftMetadata,
      collectionMint: collectionMint.publicKey,
      authority: umi.payer,
    }).sendAndConfirm(umi);

    console.log("✨ Swapping...");
    const swapNFT = await program.methods.swapNftToToken().accounts({
      sponsor: sponsorPDA,
      tokenMint: tokenMint,
      nftToken: new anchor.web3.PublicKey(publicKey(swapNftToken)),
      nftMint: new anchor.web3.PublicKey(publicKey(swapNftMint)),
      nftMetadata: new anchor.web3.PublicKey(publicKey(swapNftMetadata)),
      nftAuthority: nftAuthorityPda,
      nftCustody: nftCustody2,
      nftEdition: new anchor.web3.PublicKey(publicKey(swapNftEdition)),
      payer: signerKp.publicKey,
      sourceTokenRecord: sourceTokenRecordPubkey2,
      destinationTokenRecord: destinationTokenRecordPubkey2,
      payerTokenAccount: payerTokenAccount.address,
      sponsorTokenAccount: sponsorTokenAccount.address,
      feeWallet: new anchor.web3.PublicKey(feeWallets[0]),
      feeWalletTwo: new anchor.web3.PublicKey(feeWallets[1]),
      feeWalletThree: new anchor.web3.PublicKey(feeWallets[2]),
      metadataProgram: metadataProgram,
      systemProgram: systemProgram,
      tokenProgram: tokenProgram,
      associatedTokenProgram: associatedTokenProgram,
      sysvarInstructions: sysvarInstructions,
    })
    .preInstructions([
      anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
        units: 400_000,
      }),
    ])
    .rpc();
    console.log("✅ NFT swapped to token! Signature:", swapNFT);

    const account = await program.account.sponsor.fetch(sponsorPDA);
    const nftCustodyBalance =
        await provider.connection.getTokenAccountBalance(nftCustody);

    assert.equal(nftCustodyBalance.value.uiAmount, 1)
  });

  it("Swap Token to NFT", async () => {
    console.log("✨ Fetching sponsor token account...");
    let sponsorTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
      provider.connection,
      signerKp,
      tokenMint,
      sponsorPDA,
      true
    );

    console.log("✨ Fetching payer token account...");
    let payerTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
      provider.connection,
      signerKp,
      tokenMint,
      signerKp.publicKey,
      false,
    );

    console.log("✨ Swapping...");
    const swapNFT = await program.methods.swapTokenToNft(1).accounts({
      sponsor: sponsorPDA,
      tokenMint: tokenMint,
      nftToken: nftTokenPubkey,
      nftMint: nftMintPubkey,
      nftMetadata: nftMetadataPubkey,
      nftAuthority: nftAuthorityPda,
      nftCustody: nftCustody,
      nftEdition: nftEditionPubkey,
      payer: signerKp.publicKey,
      sourceTokenRecord: destinationTokenRecordPubkey,
      destinationTokenRecord: sourceTokenRecordPubkey,
      payerTokenAccount: payerTokenAccount.address,
      sponsorTokenAccount: sponsorTokenAccount.address,
      feeWallet: new anchor.web3.PublicKey(feeWallets[0]),
      feeWalletTwo: new anchor.web3.PublicKey(feeWallets[1]),
      feeWalletThree: new anchor.web3.PublicKey(feeWallets[2]),
      metadataProgram: metadataProgram,
      systemProgram: systemProgram,
      tokenProgram: tokenProgram,
      associatedTokenProgram: associatedTokenProgram,
      sysvarInstructions: sysvarInstructions,
    })
    .preInstructions([
      anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
        units: 400_000,
      }),
    ])
    .rpc({ skipPreflight: true });
    console.log("✅ Token swapped to NFT! Signature:", swapNFT);

    const account = await program.account.sponsor.fetch(sponsorPDA);
    //const nftCustodyBalance =
        //await provider.connection.getTokenAccountBalance(nftCustody);
    //const sponsorBalance =
        //await provider.connection.getTokenAccountBalance(sponsorTokenAccount.address);

    //assert.equal(nftCustodyBalance.value.uiAmount, 0);
    console.log("NFTs Held:", account.nftsHeld.toNumber());
    //assert.equal(sponsorBalance.value.uiAmount, 1);
  });
});
