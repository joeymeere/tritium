import * as anchor from "@coral-xyz/anchor";
import { MPL_TOKEN_METADATA_PROGRAM_ID } from "@metaplex-foundation/mpl-token-metadata";
import { SPL_SYSTEM_PROGRAM_ID, SPL_TOKEN_PROGRAM_ID } from "@metaplex-foundation/mpl-toolbox";
import { ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";

export const tokenProgram = new anchor.web3.PublicKey(
    SPL_TOKEN_PROGRAM_ID
);

export const metadataProgram = new anchor.web3.PublicKey(
    MPL_TOKEN_METADATA_PROGRAM_ID
);

export const associatedTokenProgram = new anchor.web3.PublicKey(
    ASSOCIATED_TOKEN_PROGRAM_ID
);

export const systemProgram = new anchor.web3.PublicKey(
    SPL_SYSTEM_PROGRAM_ID
);

export const sysvarInstructions = new anchor.web3.PublicKey(
    anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY
);

export const feeWallets = [
    "EYNsuoUh4pRCpuNqj5cH8zUDXST4o8YYqRg6vraG7Br7", 
    "3nHNJd8mjZFTVkA2dPTSCnWjzJU3XvC5nGSrDMWNKpQb", 
    "ghosnnrbJRNUueziNL579JZCqvcLpdHSMXU2zn9uGJS"
];