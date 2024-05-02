import {
  createSignerFromKeypair,
  signerIdentity,
} from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { mplTokenMetadata } from "@metaplex-foundation/mpl-token-metadata";

export function initUmi(rpc: string, secretKey: Uint8Array) {
  try {
    const umi = createUmi(rpc).use(mplTokenMetadata());

    const signer = umi.eddsa.createKeypairFromSecretKey(
      new Uint8Array(secretKey)
    );

    umi.use(signerIdentity(createSignerFromKeypair(umi, signer)));

    return umi;
  } catch (err) {
    throw err;
  }
}
