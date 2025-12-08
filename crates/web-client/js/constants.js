export const WorkerAction = Object.freeze({
  INIT: "init",
  CALL_METHOD: "callMethod",
  EXECUTE_CALLBACK: "executeCallback",
});

export const CallbackType = Object.freeze({
  GET_KEY: "getKey",
  INSERT_KEY: "insertKey",
  SIGN: "sign",
});

export const MethodName = Object.freeze({
  CREATE_CLIENT: "createClient",
  NEW_WALLET: "newWallet",
  NEW_FAUCET: "newFaucet",
  EXECUTE_TRANSACTION: "executeTransaction",
  PROVE_TRANSACTION: "proveTransaction",
  SUBMIT_NEW_TRANSACTION: "submitNewTransaction",
  SUBMIT_NEW_TRANSACTION_MOCK: "submitNewTransactionMock",
  SYNC_STATE: "syncState",
  SYNC_STATE_MOCK: "syncStateMock",
});
