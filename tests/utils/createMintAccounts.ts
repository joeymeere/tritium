import { Umi, generateSigner, publicKey } from "@metaplex-foundation/umi";
import {
  findMasterEditionPda,
  findMetadataPda,
} from "@metaplex-foundation/mpl-token-metadata";
import { findAssociatedTokenPda } from "@metaplex-foundation/mpl-toolbox";

export function createMintAccounts(umi: Umi) {
  try {
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

    return {
      nftMint: nftMintPubkey,
      nftToken: nftTokenPubkey,
      nftMetadata: nftMetadataPubkey,
      nftEdition: nftEditionPubkey,
    };
  } catch (err) {
    throw err;
  }
}
