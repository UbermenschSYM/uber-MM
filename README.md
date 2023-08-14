# uber-MM

clone the repo, navigate to uberMMSDK file and install npm packages
```bash
git clone git@github.com:UbermenschSYM/uber-MM.git
cd uber-MM
cd uberMMSDK
npm install
```
open test.ts file.

```typescript
let marketAddress = new PublicKey("MARKET_ADDRESS_PUBKEY");               // write market address here

  let connection = new Connection("https://api.mainnet-beta.solana.com"); // you can change RPC URL

  let uberMmSDK = await UberMmSDK.init(account, connection);              // initializing uber-mm SDK

  let baseTokenAccount = new PublicKey("YOUR_BASE_TOKEN_ACCOUNT");        // your base token account
  let quoteTokenAccount = new PublicKey("YOUR_QUOTE_TOKEN_ACCOUNT");      // your quote token account

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
```
and then just run it
```bash
ts-node test.ts
```

npm package will be published soon.
