import * as anchor from "@coral-xyz/anchor";
import { readFileSync } from "fs";
import path from "path";
import { SystemProgram } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { HybridDefi } from "../target/types/hybrid_defi";
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

describe("hybrid-defi", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(anchor.AnchorProvider.env());

  const connection = new anchor.web3.Connection(anchor.web3.clusterApiUrl());

  const program = anchor.workspace.HybridDefi as Program<HybridDefi>;

  console.log(connection);

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
      [utf8.encode("hybrid_sponsor"), program.provider.publicKey.toBuffer(), collectionMintPubkey.toBuffer()],
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

  before(async () => {
    console.log("Airdropping...");

    await umi.rpc.airdrop(umi.payer.publicKey, sol(10));

    console.log("Creating collection NFT...");
    await createNft(umi, {
      mint: collectionMint,
      name: "Test Collection",
      uri: "https://example.xyz",
      sellerFeeBasisPoints: percentAmount(5),
      isCollection: true,
    }).sendAndConfirm(umi);

    console.log("Creating a pNFT...");
    await createProgrammableNft(umi, {
      mint: nftMint,
      tokenOwner: umi.identity.publicKey,
      name: "Test pNFT",
      uri: "https://example.xyz",
      sellerFeeBasisPoints: percentAmount(5),
      collection: some({ 
        key: collectionMint.publicKey, 
        verified: false 
      }),
    }).sendAndConfirm(umi);

    console.log("Verifying...");
    await verifyCollectionV1(umi, {
      metadata: nftMetadata,
      collectionMint: collectionMint.publicKey,
      authority: umi.payer,
    }).sendAndConfirm(umi);
  });

  let payer = provider.wallet.publicKey;

    const tokenMint = await createMint(
      anchor.AnchorProvider.env().connection,
      provider.wallet,
      payer,
      payer,
      6
    );

    await mintTo(
      anchor.AnchorProvider.env().connection,
      payer,
      tokenMint,
      payer.publicKey,
      payer.publicKey,
      1000
    );

    const tx = await program.methods.initializeSponsorPool(
      tokenMint,
      nftMintPubkey,
      new anchor.BN(1000),
    ).accounts({
      hybridVault: sponsorPDA,
      collectionMint: collectionMintPubkey,
      nftAuthority: nftAuthorityPda,
      payer: payer.publicKey,
      systemProgram: systemProgram
    }).rpc();
    console.log("Your transaction signature", tx);

  it("Swap NFT to Token", async () => {
    const tx = await program.methods.swapNftToToken().accounts({
      sponsor: sponsorPDA,
      nftToken: nftTokenPubkey,
      nftMint: nftMint[0],
      nftMetadata: nftMetadata[0],
      nftAuthority: nftAuthorityPda,
      nftCustody: nftCustody,
      nftEdition: nftEditionPubkey,
      payer: payer.publicKey,
      systemProgram: systemProgram,
      tokenProgram: tokenProgram,
    }).rpc();
    console.log("Your transaction signature", tx);

    assert.exists(tx);
  });

  it("Create Mint & Init Sponsor", async () => {
    const payer = anchor.web3.Keypair.generate();

    // Fund payer account
    await connection.requestAirdrop(payer.publicKey, 2);

    const tokenMint = await createMint(
      anchor.AnchorProvider.env().connection,
      payer,
      payer.publicKey,
      payer.publicKey,
      6
    );

    await mintTo(
      anchor.AnchorProvider.env().connection,
      payer,
      tokenMint,
      payer.publicKey,
      payer.publicKey,
      1000
    );

    const tx = await program.methods.initializeSponsorPool(
      tokenMint,
      nftMintPubkey,
      new anchor.BN(1000),
    ).accounts({
      hybridVault: sponsorPDA,
      collectionMint: collectionMintPubkey,
      nftAuthority: nftAuthorityPda,
      payer: payer.publicKey,
      systemProgram: systemProgram
    }).rpc();
    console.log("Your transaction signature", tx);

    assert.exists(tx);
  });
});
