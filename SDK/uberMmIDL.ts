export type UberMm = {
    "version": "0.1.0",
    "name": "uber_mm",
    "instructions": [
      {
        "name": "initialize",
        "accounts": [
          {
            "name": "phoenixStrategy",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "user",
            "isMut": true,
            "isSigner": true
          },
          {
            "name": "market",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "systemProgram",
            "isMut": false,
            "isSigner": false
          }
        ],
        "args": [
          {
            "name": "quoteEdgeInBps",
            "type": "u64"
          },
          {
            "name": "quoteSizeInQuoteAtoms",
            "type": "u64"
          },
          {
            "name": "priceImprovementBehavior",
            "type": "u8"
          },
          {
            "name": "postOnly",
            "type": "bool"
          }
        ]
      },
      {
        "name": "updateQuotes",
        "accounts": [
          {
            "name": "phoenixStrategy",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "user",
            "isMut": false,
            "isSigner": true
          },
          {
            "name": "phoenixProgram",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "logAuthority",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "market",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "seat",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "quoteAccount",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "baseAccount",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "quoteVault",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "baseVault",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "tokenProgram",
            "isMut": false,
            "isSigner": false
          }
        ],
        "args": [
          {
            "name": "fairPriceInQuoteAtomsPerRawBaseUnit",
            "type": "u64"
          },
          {
            "name": "quoteEdgeInBps",
            "type": "u64"
          },
          {
            "name": "quoteSizeInQuoteAtoms",
            "type": "u64"
          },
          {
            "name": "priceImprovementBehavior",
            "type": "u8"
          },
          {
            "name": "postOnly",
            "type": "bool"
          },
          {
            "name": "useOracle",
            "type": "bool"
          },
          {
            "name": "margin",
            "type": "u64"
          }
        ]
      }
    ],
    "accounts": [
      {
        "name": "phoenixStrategyState",
        "type": {
          "kind": "struct",
          "fields": [
            {
              "name": "trader",
              "type": "publicKey"
            },
            {
              "name": "market",
              "type": "publicKey"
            },
            {
              "name": "bidOrderSequenceNumber",
              "type": "u64"
            },
            {
              "name": "bidPriceInTicks",
              "type": "u64"
            },
            {
              "name": "initialBidSizeInBaseLots",
              "type": "u64"
            },
            {
              "name": "askOrderSequenceNumber",
              "type": "u64"
            },
            {
              "name": "askPriceInTicks",
              "type": "u64"
            },
            {
              "name": "initialAskSizeInBaseLots",
              "type": "u64"
            },
            {
              "name": "lastUpdateSlot",
              "type": "u64"
            },
            {
              "name": "lastUpdateUnixTimestamp",
              "type": "i64"
            },
            {
              "name": "quoteEdgeInBps",
              "docs": [
                "Number of basis points betweeen quoted price and fair price"
              ],
              "type": "u64"
            },
            {
              "name": "quoteSizeInQuoteAtoms",
              "docs": [
                "Order notional size in quote atoms"
              ],
              "type": "u64"
            },
            {
              "name": "postOnly",
              "docs": [
                "If set to true, the orders will never cross the spread"
              ],
              "type": "bool"
            },
            {
              "name": "priceImprovementBehavior",
              "docs": [
                "Determines whether/how to improve BBO"
              ],
              "type": "u8"
            },
            {
              "name": "padding",
              "type": {
                "array": [
                  "u8",
                  6
                ]
              }
            }
          ]
        }
      }
    ],
    "types": [
      {
        "name": "OrderParams",
        "type": {
          "kind": "struct",
          "fields": [
            {
              "name": "fairPriceInQuoteAtomsPerRawBaseUnit",
              "type": "u64"
            },
            {
              "name": "strategyParams",
              "type": {
                "defined": "StrategyParams"
              }
            },
            {
              "name": "useOracle",
              "type": "bool"
            }
          ]
        }
      },
      {
        "name": "StrategyParams",
        "type": {
          "kind": "struct",
          "fields": [
            {
              "name": "quoteEdgeInBps",
              "type": {
                "option": "u64"
              }
            },
            {
              "name": "quoteSizeInQuoteAtoms",
              "type": {
                "option": "u64"
              }
            },
            {
              "name": "priceImprovementBehavior",
              "type": {
                "option": {
                  "defined": "PriceImprovementBehavior"
                }
              }
            },
            {
              "name": "postOnly",
              "type": {
                "option": "bool"
              }
            }
          ]
        }
      },
      {
        "name": "PriceStatus",
        "type": {
          "kind": "enum",
          "variants": [
            {
              "name": "Unknown"
            },
            {
              "name": "Trading"
            },
            {
              "name": "Halted"
            },
            {
              "name": "Auction"
            }
          ]
        }
      },
      {
        "name": "CorpAction",
        "type": {
          "kind": "enum",
          "variants": [
            {
              "name": "NoCorpAct"
            }
          ]
        }
      },
      {
        "name": "PriceType",
        "type": {
          "kind": "enum",
          "variants": [
            {
              "name": "Unknown"
            },
            {
              "name": "Price"
            },
            {
              "name": "TWAP"
            },
            {
              "name": "Volatility"
            }
          ]
        }
      },
      {
        "name": "PriceImprovementBehavior",
        "type": {
          "kind": "enum",
          "variants": [
            {
              "name": "Ubermensch"
            },
            {
              "name": "Join"
            },
            {
              "name": "Dime"
            },
            {
              "name": "Ignore"
            }
          ]
        }
      }
    ],
    "errors": [
      {
        "code": 6000,
        "name": "NoReturnData"
      },
      {
        "code": 6001,
        "name": "InvalidStrategyParams"
      },
      {
        "code": 6002,
        "name": "EdgeMustBeNonZero"
      },
      {
        "code": 6003,
        "name": "InvalidPhoenixProgram"
      },
      {
        "code": 6004,
        "name": "FailedToDeserializePhoenixMarket"
      },
      {
        "code": 6005,
        "name": "PythStatus"
      },
      {
        "code": 6006,
        "name": "PythValidSlot"
      },
      {
        "code": 6007,
        "name": "PythNegativePrice"
      },
      {
        "code": 6008,
        "name": "PythConfidence"
      }
    ]
  };
  
  export const UberMmIDL: UberMm = {
    "version": "0.1.0",
    "name": "uber_mm",
    "instructions": [
      {
        "name": "initialize",
        "accounts": [
          {
            "name": "phoenixStrategy",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "user",
            "isMut": true,
            "isSigner": true
          },
          {
            "name": "market",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "systemProgram",
            "isMut": false,
            "isSigner": false
          }
        ],
        "args": [
          {
            "name": "quoteEdgeInBps",
            "type": "u64"
          },
          {
            "name": "quoteSizeInQuoteAtoms",
            "type": "u64"
          },
          {
            "name": "priceImprovementBehavior",
            "type": "u8"
          },
          {
            "name": "postOnly",
            "type": "bool"
          }
        ]
      },
      {
        "name": "updateQuotes",
        "accounts": [
          {
            "name": "phoenixStrategy",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "user",
            "isMut": false,
            "isSigner": true
          },
          {
            "name": "phoenixProgram",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "logAuthority",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "market",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "seat",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "quoteAccount",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "baseAccount",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "quoteVault",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "baseVault",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "tokenProgram",
            "isMut": false,
            "isSigner": false
          }
        ],
        "args": [
          {
            "name": "fairPriceInQuoteAtomsPerRawBaseUnit",
            "type": "u64"
          },
          {
            "name": "quoteEdgeInBps",
            "type": "u64"
          },
          {
            "name": "quoteSizeInQuoteAtoms",
            "type": "u64"
          },
          {
            "name": "priceImprovementBehavior",
            "type": "u8"
          },
          {
            "name": "postOnly",
            "type": "bool"
          },
          {
            "name": "useOracle",
            "type": "bool"
          },
          {
            "name": "margin",
            "type": "u64"
          }
        ]
      }
    ],
    "accounts": [
      {
        "name": "phoenixStrategyState",
        "type": {
          "kind": "struct",
          "fields": [
            {
              "name": "trader",
              "type": "publicKey"
            },
            {
              "name": "market",
              "type": "publicKey"
            },
            {
              "name": "bidOrderSequenceNumber",
              "type": "u64"
            },
            {
              "name": "bidPriceInTicks",
              "type": "u64"
            },
            {
              "name": "initialBidSizeInBaseLots",
              "type": "u64"
            },
            {
              "name": "askOrderSequenceNumber",
              "type": "u64"
            },
            {
              "name": "askPriceInTicks",
              "type": "u64"
            },
            {
              "name": "initialAskSizeInBaseLots",
              "type": "u64"
            },
            {
              "name": "lastUpdateSlot",
              "type": "u64"
            },
            {
              "name": "lastUpdateUnixTimestamp",
              "type": "i64"
            },
            {
              "name": "quoteEdgeInBps",
              "docs": [
                "Number of basis points betweeen quoted price and fair price"
              ],
              "type": "u64"
            },
            {
              "name": "quoteSizeInQuoteAtoms",
              "docs": [
                "Order notional size in quote atoms"
              ],
              "type": "u64"
            },
            {
              "name": "postOnly",
              "docs": [
                "If set to true, the orders will never cross the spread"
              ],
              "type": "bool"
            },
            {
              "name": "priceImprovementBehavior",
              "docs": [
                "Determines whether/how to improve BBO"
              ],
              "type": "u8"
            },
            {
              "name": "padding",
              "type": {
                "array": [
                  "u8",
                  6
                ]
              }
            }
          ]
        }
      }
    ],
    "types": [
      {
        "name": "OrderParams",
        "type": {
          "kind": "struct",
          "fields": [
            {
              "name": "fairPriceInQuoteAtomsPerRawBaseUnit",
              "type": "u64"
            },
            {
              "name": "strategyParams",
              "type": {
                "defined": "StrategyParams"
              }
            },
            {
              "name": "useOracle",
              "type": "bool"
            }
          ]
        }
      },
      {
        "name": "StrategyParams",
        "type": {
          "kind": "struct",
          "fields": [
            {
              "name": "quoteEdgeInBps",
              "type": {
                "option": "u64"
              }
            },
            {
              "name": "quoteSizeInQuoteAtoms",
              "type": {
                "option": "u64"
              }
            },
            {
              "name": "priceImprovementBehavior",
              "type": {
                "option": {
                  "defined": "PriceImprovementBehavior"
                }
              }
            },
            {
              "name": "postOnly",
              "type": {
                "option": "bool"
              }
            }
          ]
        }
      },
      {
        "name": "PriceStatus",
        "type": {
          "kind": "enum",
          "variants": [
            {
              "name": "Unknown"
            },
            {
              "name": "Trading"
            },
            {
              "name": "Halted"
            },
            {
              "name": "Auction"
            }
          ]
        }
      },
      {
        "name": "CorpAction",
        "type": {
          "kind": "enum",
          "variants": [
            {
              "name": "NoCorpAct"
            }
          ]
        }
      },
      {
        "name": "PriceType",
        "type": {
          "kind": "enum",
          "variants": [
            {
              "name": "Unknown"
            },
            {
              "name": "Price"
            },
            {
              "name": "TWAP"
            },
            {
              "name": "Volatility"
            }
          ]
        }
      },
      {
        "name": "PriceImprovementBehavior",
        "type": {
          "kind": "enum",
          "variants": [
            {
              "name": "Ubermensch"
            },
            {
              "name": "Join"
            },
            {
              "name": "Dime"
            },
            {
              "name": "Ignore"
            }
          ]
        }
      }
    ],
    "errors": [
      {
        "code": 6000,
        "name": "NoReturnData"
      },
      {
        "code": 6001,
        "name": "InvalidStrategyParams"
      },
      {
        "code": 6002,
        "name": "EdgeMustBeNonZero"
      },
      {
        "code": 6003,
        "name": "InvalidPhoenixProgram"
      },
      {
        "code": 6004,
        "name": "FailedToDeserializePhoenixMarket"
      },
      {
        "code": 6005,
        "name": "PythStatus"
      },
      {
        "code": 6006,
        "name": "PythValidSlot"
      },
      {
        "code": 6007,
        "name": "PythNegativePrice"
      },
      {
        "code": 6008,
        "name": "PythConfidence"
      }
    ]
  };
  