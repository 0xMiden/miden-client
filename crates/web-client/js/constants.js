export const WorkerAction = Object.freeze({
  INIT: "init",
  CALL_METHOD: "callMethod",
});

export const MethodName = Object.freeze({
  CREATE_CLIENT: "createClient",
  NEW_WALLET: "newWallet",
  NEW_FAUCET: "newFaucet",
  EXECUTE_TRANSACTION: "executeTransaction",
  PROVE_TRANSACTION: "proveTransaction",
  SUBMIT_NEW_TRANSACTION: "submitNewTransaction",
  SUBMIT_NEW_TRANSACTION_MOCK: "submitNewTransactionMock",
  SUBMIT_NEW_TRANSACTION_WITH_PROVER: "submitNewTransactionWithProver",
  SUBMIT_NEW_TRANSACTION_WITH_PROVER_MOCK: "submitNewTransactionWithProverMock",
  SYNC_STATE: "syncState",
  SYNC_STATE_MOCK: "syncStateMock",
});
