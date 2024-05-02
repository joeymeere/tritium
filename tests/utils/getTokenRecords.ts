import { PublicKey, Umi, publicKey } from "@metaplex-foundation/umi";
import { findTokenRecordPda } from "@metaplex-foundation/mpl-token-metadata";

export function getTokenRecords(
  umi: Umi,
  keys: {
    nftMint: PublicKey;
    nftToken: PublicKey;
    nftCustody: PublicKey;
  }
) {
  try {
    const sourceTokenRecord = findTokenRecordPda(umi, {
      mint: keys.nftMint,
      token: publicKey(keys.nftToken),
    });
    const sourceTokenRecordPubkey = new anchor.web3.PublicKey(
      publicKey(sourceTokenRecord)
    );

    const destinationTokenRecord = findTokenRecordPda(umi, {
      mint: keys.nftMint,
      token: publicKey(keys.nftCustody),
    });

    const destinationTokenRecordPubkey = new anchor.web3.PublicKey(
      publicKey(destinationTokenRecord)
    );

    return {
      sourceTokenRecord: sourceTokenRecordPubkey,
      destinationTokenRecord: destinationTokenRecordPubkey,
    };
  } catch (err) {
    throw err;
  }
}
