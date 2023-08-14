import { TokenConfig } from "@ellipsis-labs/phoenix-sdk";
import { Connection, Keypair, PublicKey, Transaction } from "@solana/web3.js";

export const UBER_MM_PROGRAM_ID = new PublicKey("Exz7z8HpBjS7trD6ZbdWABdQyhK5ZvGkuV4UYoUiSTQQ");
export const PHOENIX_PROGRAM_ID = new PublicKey("PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY");

export const tokenConfig: Map<string, TokenConfig> = new Map([
    [
      "USDC",
      {
        name: "USD Coin",
        symbol: "USDC",
        mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        logoUri:
          "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png",
      },
    ],
    [
      "SOL",
      {
        name: "Wrapped SOL",
        symbol: "SOL",
        mint: "So11111111111111111111111111111111111111112",
        logoUri:
          "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
      },
    ],
  ]);

export interface Wallet {
    signTransaction(tx: Transaction): Promise<Transaction>;
    signAllTransactions(txs: Transaction[]): Promise<Transaction[]>;
    publicKey: PublicKey;
}