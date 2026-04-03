// @ts-nocheck
import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";

// GET_TRANSACTIONS TESTS
// =======================================================================================================

// Helper to mint and consume a transaction using the mock client, returning the transaction IDs.
const mockMintAndConsume = async (
  page: Page,
  client: any,
  accountId: any,
  faucetId: any,
  commit: boolean = true
) => {
  return await page.evaluate(
    async ({ commit }) => {
      const client = window._mockClient;
      const accountId = window._mockAccountId;
      const faucetId = window._mockFaucetId;

      const mintRequest = await client.newMintTransactionRequest(
        accountId,
        faucetId,
        window.NoteType.Public,
        BigInt(1000)
      );

      const mintTxId = await client.submitNewTransaction(faucetId, mintRequest);
      const mintTxIdHex = mintTxId.toHex();
      await client.proveBlock();
      await client.syncState();

      const [mintRecord] = await client.getTransactions(
        window.TransactionFilter.ids([mintTxId])
      );
      const mintedNoteId = mintRecord.outputNotes().notes()[0].id().toString();

      const mintedNoteRecord = await client.getInputNote(mintedNoteId);
      const mintedNote = mintedNoteRecord.toNote();
      const consumeRequest = client.newConsumeTransactionRequest([mintedNote]);

      const consumeTxId = await client.submitNewTransaction(
        accountId,
        consumeRequest
      );
      const consumeTxIdHex = consumeTxId.toHex();

      if (commit) {
        await client.proveBlock();
        await client.syncState();
      }

      return {
        mintTxId: mintTxIdHex,
        consumeTxId: consumeTxIdHex,
      };
    },
    { commit }
  );
};

// Helper to set up a mock client with a wallet and faucet, stored on window for use in evaluate calls.
const setupMockClient = async (page: Page) => {
  return await page.evaluate(async () => {
    const client = await window.MockWasmWebClient.createClient();
    await client.syncState();

    const account = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      window.AuthScheme.AuthRpoFalcon512
    );
    const faucet = await client.newFaucet(
      window.AccountStorageMode.private(),
      false,
      "DAG",
      8,
      BigInt(10000000),
      window.AuthScheme.AuthRpoFalcon512
    );

    // Store on window for use in subsequent evaluate calls
    window._mockClient = client;
    window._mockAccountId = account.id();
    window._mockFaucetId = faucet.id();

    return {
      accountId: account.id().toString(),
      faucetId: faucet.id().toString(),
    };
  });
};

const getAllTransactions = async (page: Page) => {
  return await page.evaluate(async () => {
    const client = window._mockClient;

    let transactions = await client.getTransactions(
      window.TransactionFilter.all()
    );
    let uncommittedTransactions = await client.getTransactions(
      window.TransactionFilter.uncommitted()
    );
    let transactionIds = transactions.map((transaction) =>
      transaction.id().toHex()
    );
    let uncommittedTransactionIds = uncommittedTransactions.map((transaction) =>
      transaction.id().toHex()
    );

    return {
      transactionIds,
      uncommittedTransactionIds,
    };
  });
};

test.describe("get_transactions tests", () => {
  test("get_transactions retrieves all transactions successfully", async ({
    page,
  }) => {
    await setupMockClient(page);
    const { mintTxId, consumeTxId } = await mockMintAndConsume(
      page,
      null,
      null,
      null
    );

    const result = await getAllTransactions(page);

    expect(result.transactionIds).toContain(mintTxId);
    expect(result.transactionIds).toContain(consumeTxId);
    expect(result.uncommittedTransactionIds.length).toEqual(0);
  });

  test("get_transactions retrieves uncommitted transactions successfully", async ({
    page,
  }) => {
    await setupMockClient(page);

    // Committed mint+consume
    const { mintTxId, consumeTxId } = await mockMintAndConsume(
      page,
      null,
      null,
      null
    );

    // Uncommitted mint (no proveBlock/sync)
    const uncommittedTxId = await page.evaluate(async () => {
      const client = window._mockClient;
      const accountId = window._mockAccountId;
      const faucetId = window._mockFaucetId;

      const mintRequest = await client.newMintTransactionRequest(
        accountId,
        faucetId,
        window.NoteType.Public,
        BigInt(1000)
      );

      const txId = await client.submitNewTransaction(faucetId, mintRequest);
      return txId.toHex();
    });

    const result = await getAllTransactions(page);

    expect(result.transactionIds).toContain(mintTxId);
    expect(result.transactionIds).toContain(consumeTxId);
    expect(result.transactionIds).toContain(uncommittedTxId);
    expect(result.transactionIds.length).toEqual(3);

    expect(result.uncommittedTransactionIds).toContain(uncommittedTxId);
    expect(result.uncommittedTransactionIds.length).toEqual(1);
  });

  test("get_transactions retrieves no transactions successfully", async ({
    page,
  }) => {
    await setupMockClient(page);
    const result = await getAllTransactions(page);

    expect(result.transactionIds.length).toEqual(0);
    expect(result.uncommittedTransactionIds.length).toEqual(0);
  });

  test("get_transactions filters by specific transaction IDs successfully", async ({
    page,
  }) => {
    await setupMockClient(page);
    await mockMintAndConsume(page, null, null, null);

    const result = await page.evaluate(async () => {
      const client = window._mockClient;

      let allTransactions = await client.getTransactions(
        window.TransactionFilter.all()
      );
      const allTxLength = allTransactions.length;
      let firstTransactionId = allTransactions[0].id();
      const firstTxIdHex = firstTransactionId.toHex();

      const filter = window.TransactionFilter.ids([firstTransactionId]);
      let filteredTransactions = await client.getTransactions(filter);
      const filteredTransactionIds = filteredTransactions.map((tx) =>
        tx.id().toHex()
      );

      return {
        allTransactionsCount: allTxLength,
        filteredTransactionIds,
        originalTransactionId: firstTxIdHex,
      };
    });

    expect(result.allTransactionsCount).toEqual(2);
    expect(result.filteredTransactionIds.length).toEqual(1);
    expect(result.filteredTransactionIds).toContain(
      result.originalTransactionId
    );
  });

  test("get_transactions filters expired transactions successfully", async ({
    page,
  }) => {
    await setupMockClient(page);

    // Committed mint
    const committedTxId = await page.evaluate(async () => {
      const client = window._mockClient;
      const accountId = window._mockAccountId;
      const faucetId = window._mockFaucetId;

      const mintRequest = await client.newMintTransactionRequest(
        accountId,
        faucetId,
        window.NoteType.Public,
        BigInt(1000)
      );

      const txId = await client.submitNewTransaction(faucetId, mintRequest);
      await client.proveBlock();
      await client.syncState();
      return txId.toHex();
    });

    // Uncommitted mint
    const uncommittedTxId = await page.evaluate(async () => {
      const client = window._mockClient;
      const accountId = window._mockAccountId;
      const faucetId = window._mockFaucetId;

      const mintRequest = await client.newMintTransactionRequest(
        accountId,
        faucetId,
        window.NoteType.Public,
        BigInt(1000)
      );

      const txId = await client.submitNewTransaction(faucetId, mintRequest);
      return txId.toHex();
    });

    const result = await page.evaluate(async () => {
      const client = window._mockClient;

      let allTransactions = await client.getTransactions(
        window.TransactionFilter.all()
      );
      let allTransactionIds = allTransactions.map((tx) => tx.id().toHex());
      let currentBlockNum = allTransactions[0].blockNum();

      let futureBlockNum = currentBlockNum + 100;
      let futureExpiredFilter =
        window.TransactionFilter.expiredBefore(futureBlockNum);
      let futureExpiredTransactions =
        await client.getTransactions(futureExpiredFilter);
      let futureExpiredTransactionIds = futureExpiredTransactions.map((tx) =>
        tx.id().toHex()
      );

      let pastBlockNum = currentBlockNum - 10;
      let pastExpiredFilter =
        window.TransactionFilter.expiredBefore(pastBlockNum);
      let pastExpiredTransactions =
        await client.getTransactions(pastExpiredFilter);
      let pastExpiredTransactionIds = pastExpiredTransactions.map((tx) =>
        tx.id().toHex()
      );

      return {
        allTransactionIds,
        futureExpiredTransactionIds,
        pastExpiredTransactionIds,
      };
    });

    expect(result.futureExpiredTransactionIds.length).toEqual(1);
    expect(result.futureExpiredTransactionIds).toContain(uncommittedTxId);
    expect(result.pastExpiredTransactionIds.length).toEqual(0);
    expect(result.allTransactionIds.length).toEqual(2);
    expect(result.allTransactionIds).toContain(committedTxId);
    expect(result.allTransactionIds).toContain(uncommittedTxId);
  });
});

// COMPILE_TX_SCRIPT TESTS
// =======================================================================================================

interface CompileTxScriptResult {
  scriptRoot: string;
}

export const compileTxScript = async (
  page: Page,
  script: string
): Promise<CompileTxScriptResult> => {
  return await page.evaluate(async (_script: string) => {
    const client = window.client;

    let walletAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true,
      window.AuthScheme.AuthRpoFalcon512
    );

    let builder = client.createCodeBuilder();
    const compiledScript = builder.compileNoteScript(_script);

    return {
      scriptRoot: compiledScript.root().toHex(),
    };
  }, script);
};

test.describe("compile_tx_script tests", () => {
  test("compile_tx_script compiles script successfully", async ({ page }) => {
    const script = `
            begin
                push.0 push.0
                # => [0, 0]
                assert_eq
            end
        `;
    const result = await compileTxScript(page, script);

    expect(result.scriptRoot.length).toBeGreaterThan(1);
  });

  test("compile_tx_script does not compile script successfully", async ({
    page,
  }) => {
    const script = "fakeScript";

    await expect(compileTxScript(page, script)).rejects.toThrow(
      /failed to compile note script:/
    );
  });
});
