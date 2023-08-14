import { AnchorProvider, Program } from "@coral-xyz/anchor";
import * as anchor from "@coral-xyz/anchor";
import { UberMm, UberMmIDL } from "./uberMmIDL";
import { PublicKey, Connection, Keypair, SystemProgram, Account, sendAndConfirmTransaction, Transaction } from "@solana/web3.js";
import { PHOENIX_PROGRAM_ID, UBER_MM_PROGRAM_ID } from "./consts";
import { createPhoenixClient, getTokenBalance } from "./helpers";
import * as Phoenix from "@ellipsis-labs/phoenix-sdk";
import { Wallet } from "./consts";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { BN } from "bn.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { parsePriceData } from "@pythnetwork/client";
import { marketsToPyth } from "./marketsToPyth";
export class UberMmSDK {
    private wallet: Account;
    private program: Program<UberMm>;
    private connection: Connection;

    constructor(wallet: Account, program: Program<UberMm>, connection: Connection) {
        this.wallet = wallet;
        this.program = program;
        this.connection = connection;
    }

    static async init(
        wallet: Account,
        connection: Connection,
    ): Promise<UberMmSDK> {
        const provider = new AnchorProvider(
            connection,
            new NodeWallet(new Keypair({
                publicKey: wallet.publicKey.toBuffer(),
                secretKey: wallet.secretKey,
            })),
            { commitment: "confirmed", preflightCommitment: "confirmed"}
        );
        let program = new Program(UberMmIDL, UBER_MM_PROGRAM_ID, provider);
        return new UberMmSDK(wallet, program, connection);
    }

    public async getPhoenixMarketAddresses(): Promise<PublicKey[]> {
        let programAccounts = await this.connection.getProgramAccounts(
            new PublicKey(PHOENIX_PROGRAM_ID),
            {
                commitment: this.connection.commitment,
                filters: [{dataSize: 1723488}],
                encoding: 'base64'
            }
        );
        return programAccounts.map((account: any) => {return account.pubkey});
    }

    public async initializeStrategyState(
        params: MMParams,
        marketAddress: PublicKey,
    ): Promise<string> {
        let phoenixClient = await createPhoenixClient(this.connection, marketAddress);
        const phoenixMarket = phoenixClient.marketStates.get(
            marketAddress.toBase58()
        );
        let [phoenixStrategy, _] = PublicKey.findProgramAddressSync(
            [
                anchor.utils.bytes.utf8.encode("phoenix"),
                this.wallet.publicKey.toBuffer(),
                marketAddress.toBuffer(),
            ],
            UBER_MM_PROGRAM_ID
        );
        try {
        const tx = await this.program.methods
                .initialize(
                    new BN(params.quoteEdgeInBps),
                    new BN(params.quoteSizeInQuoteAtoms),
                    0,
                    false,
                )
                .accounts({
                    phoenixStrategy: phoenixStrategy,
                    user: this.wallet.publicKey,
                    market: marketAddress,
                    systemProgram: SystemProgram.programId,
                })
                .signers([this.wallet])
                .rpc();
            } catch(e){console.log(e)}
        try {
            let seatTx = phoenixMarket.createRequestSeatInstruction(this.wallet.publicKey, this.wallet.publicKey);
            let res = await sendAndConfirmTransaction(this.connection, new Transaction().add(seatTx), [this.wallet]); } catch(e){console.log(e)}
        try {
            let claimSeatTx = await Phoenix.getClaimSeatIx(marketAddress, this.wallet.publicKey);
            let res = await sendAndConfirmTransaction(this.connection, new Transaction().add(claimSeatTx), [this.wallet]);
        }catch(e){console.log(e)}
        return;
    }

    public async runUberMM(
        params: MMParams, 
        marketAddress: PublicKey,
        makerQuoteTokenAccount: PublicKey,
        makerBaseTokenAccount: PublicKey,
        txNumber: number, // how many txs do you want to make
        txInterval: number, // how many seconds between each tx
    ) {
        let [phoenixStrategy, _] = PublicKey.findProgramAddressSync(
            [
                anchor.utils.bytes.utf8.encode("phoenix"),
                this.wallet.publicKey.toBuffer(),
                marketAddress.toBuffer(),
            ],
            UBER_MM_PROGRAM_ID
        );
        let phoenixClient = await createPhoenixClient(this.connection, marketAddress);
        const phoenixMarket = phoenixClient.marketStates.get(
            marketAddress.toBase58()
        );

        let baseDecimals = 10 ** phoenixMarket.data.header.baseParams.decimals;
        let quoteDecimals = 10 ** phoenixMarket.data.header.quoteParams.decimals;

        let quoteStartBalance = await getTokenBalance(this.connection, makerQuoteTokenAccount) / quoteDecimals;
        let baseStartBalance = await getTokenBalance(this.connection, makerBaseTokenAccount) / baseDecimals;
        

        console.log("BaseBalance:", baseStartBalance, "QuoteBalance: ", quoteStartBalance);
        

        for(let i = 0; i < txNumber; i++){
            let timeStart = Date.now();
            try {
                const tx = await this.program.methods
                  .updateQuotes(
                    new BN(0),
                    new BN(params.quoteEdgeInBps),
                    new BN(params.quoteSizeInQuoteAtoms),
                    params.priceImprovementBehavior,
                    false,
                    true,
                    new BN(params.margin),
                  )
                  .accounts({
                    user: this.wallet.publicKey,
                    market: marketAddress,
                    phoenixProgram: Phoenix.PROGRAM_ID,
                    phoenixStrategy: phoenixStrategy,
                    logAuthority: Phoenix.getLogAuthority(),
                    seat: phoenixMarket.getSeatAddress(this.wallet.publicKey),
                    quoteAccount: makerQuoteTokenAccount,
                    baseAccount: makerBaseTokenAccount,
                    quoteVault: phoenixMarket.data.header.quoteParams.vaultKey,
                    baseVault: phoenixMarket.data.header.baseParams.vaultKey,
                    tokenProgram: TOKEN_PROGRAM_ID,
                  }).remainingAccounts([
                    {
                        pubkey: new PublicKey(marketsToPyth[marketAddress.toBase58()][0]),
                        isWritable: false,
                        isSigner: false,
                    },
                    {
                        pubkey: new PublicKey(marketsToPyth[marketAddress.toBase58()][1]),
                        isWritable: false,
                        isSigner: false,
                    }]).signers([this.wallet]).rpc();
                // let hash = sendAndConfirmTransaction(this.connection, new Transaction().add(tx), [this.wallet]);
                console.log(i, tx);
                if(i % 10 == 9){
                  let quoteBalance = await getTokenBalance(this.connection, makerQuoteTokenAccount) / quoteDecimals;
                  let baseBalance = await getTokenBalance(this.connection, makerBaseTokenAccount) / baseDecimals;
                  console.log("BaseBalance:", baseBalance, "QuoteBalance: ", quoteBalance);
                }
            } catch(e){console.log(e)}
            let timeSpent = Date.now() - timeStart;
            let waitTime = Math.max(0, txInterval - timeSpent);
            await new Promise((r) => setTimeout(r, waitTime));
        }
        let cancelOrdersTx = phoenixMarket.createCancelAllOrdersInstruction(this.wallet.publicKey);
        console.log("canceling all orders tx: ", await sendAndConfirmTransaction(this.connection, new Transaction().add(cancelOrdersTx), [this.wallet]));

        let getTx = await phoenixMarket.createWithdrawFundsInstruction({withdrawFundsParams:{quoteLotsToWithdraw: null, baseLotsToWithdraw: null}}, this.wallet.publicKey);
        console.log("withdrawing funds tx: ", await sendAndConfirmTransaction(this.connection, new Transaction().add(getTx), [this.wallet]));
        await new Promise((r) => setTimeout(r, 5000));
        let quoteBalance = await getTokenBalance(this.connection, makerQuoteTokenAccount) / quoteDecimals;
        let baseBalance = await getTokenBalance(this.connection, makerBaseTokenAccount) / baseDecimals;

        console.log("Balances After MM: ")
        console.log("BaseBalance:", baseBalance, "QuoteBalance: ", quoteBalance);
        
        let accInfo = await this.connection.getAccountInfo(new PublicKey(marketsToPyth[marketAddress.toBase58()][0]));
        let priceBase = parsePriceData(accInfo.data).aggregate.price;
        accInfo = await this.connection.getAccountInfo(new PublicKey(marketsToPyth[marketAddress.toBase58()][1]));
        let priceQuote = parsePriceData(accInfo.data).aggregate.price;
        console.log("Profit Made: ", baseBalance * priceBase - baseStartBalance * priceBase + quoteBalance * priceQuote - quoteStartBalance * priceQuote, "USD");
    }
}

interface MMParams {
    quoteEdgeInBps: number; // edge from fair price in which we put orders
    quoteSizeInQuoteAtoms: number; // size of orders in quote atoms
    postOnly: boolean;
    priceImprovementBehavior: number; // 0 = ubermensch, 1 = join, 2 = dime, 3 = ignore
    margin: number; // minimum quote edge accepted(only used in ubermensch mode)
}