export const WorkerAction = Object.freeze({
  INIT: "init",
  CALL_METHOD: "callMethod",
});

export const MethodName = Object.freeze({
  CREATE_CLIENT: "createClient",
  NEW_WALLET: "newWallet",
  NEW_FAUCET: "newFaucet",
  SYNC_STATE: "syncState",
  SYNC_STATE_MOCK: "syncStateMock",
});
