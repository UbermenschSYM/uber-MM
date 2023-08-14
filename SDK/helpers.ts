import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, BN, Program } from "@coral-xyz/anchor";
import { UberMm } from "../target/types/uber_mm";
import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  AccountInfo,
  Account
} from "@solana/web3.js";
import * as BufferLayout from 'buffer-layout';
import { assert } from "chai";
import { deserializeAccount } from "@orca-so/sdk";
import * as Phoenix from "@ellipsis-labs/phoenix-sdk";
import { tokenConfig } from "./consts";
// WSOL mint address on the mainnet-beta network
const wsolMintAddress = new PublicKey('So11111111111111111111111111111111111111112');

export const createPhoenixClient = async (
  connection: Connection,
  marketAddress: PublicKey
): Promise<Phoenix.Client> => {
  const client = await Phoenix.Client.createWithoutConfig(connection, []);
  client.tokenConfigs = tokenConfig;
  await client.addMarket(marketAddress.toBase58());
  return client;
};


export async function getTokenBalance(connection: Connection, accountAddress: PublicKey) {
  try {

    // Fetch the account data
    const accountInfo: AccountInfo<Buffer> | null = await connection.getAccountInfo(accountAddress);
    
    if (accountInfo) {
      return deserializeAccount(accountInfo.data)?.amount.toNumber();
    } else {
      console.log('Account not found on Solana.');
    }
  } catch (error) {
    console.error('Error:', error);
  }
}

// Token Account Layout
const TokenAccountLayout = BufferLayout.struct([
  BufferLayout.u32('mint'),
  BufferLayout.u32('owner'),
  BufferLayout.nu64('amount'),
  BufferLayout.u32('state'),
]);
