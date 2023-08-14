import {PublicKey, Connection, } from "@solana/web3.js"
import { UberMmSDK } from "./uberMmSDK";
import { Account } from "@solana/web3.js";
const account = new Account([0]); // PUT YOUR PRIVATE KEY HERE
export async function test() {
    let marketAddress = new PublicKey("MARKET_ADDRESS_PUBKEY");

    let connection = new Connection("https://api.mainnet-beta.solana.com");

    let uberMmSDK = await UberMmSDK.init(account, connection);

    let baseTokenAccount = new PublicKey("YOUR_BASE_TOKEN_ACCOUNT");
    let quoteTokenAccount = new PublicKey("YOUR_QUOTE_TOKEN_ACCOUNT");

    let params = {
        quoteEdgeInBps: 3,             // edge from fair price in which we put orders
        quoteSizeInQuoteAtoms: 100000, // size of orders in quote atoms
        postOnly: false,
        priceImprovementBehavior: 0,   // 0 = ubermensch, 1 = join, 2 = dime, 3 = ignore
        margin: 3,                    // minimum quote edge accepted (only used in ubermensch mode)
    }

    // if you have not MM ed on this market before, you need to initialize the strategy state
    // this function will also handle the creation of market seat account and approving it

    await uberMmSDK.initializeStrategyState(
        params,
        marketAddress,
    );

    await uberMmSDK.runUberMM(
        params,
        marketAddress,
        quoteTokenAccount,
        baseTokenAccount,
        10,                  // number of txs sent
        1000,                // time interval between txs
    );
}

test();