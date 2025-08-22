export const WorkerAction = Object.freeze({
  INIT: "init",
  CALL_METHOD: "callMethod",
});

export const MethodName = Object.freeze({
  CREATE_CLIENT: "createClient",
  NEW_WALLET: "newWallet",
  NEW_FAUCET: "newFaucet",
  NEW_TRANSACTION: "newTransaction",
  SUBMIT_TRANSACTION: "submitTransaction",
  SUBMIT_TRANSACTION_MOCK: "submitTransactionMock",
  SYNC_STATE: "syncState",
  SYNC_STATE_MOCK: "syncStateMock",
});
