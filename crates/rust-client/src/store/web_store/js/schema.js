import Dexie from "dexie";

const DATABASE_NAME = "MidenClientDB";

export async function openDatabase() {
  console.log("Opening database...");
  try {
    await db.open();
    console.log("Database opened successfully");
    return true;
  } catch (err) {
    console.error("Failed to open database: ", err.toString());
    return false;
  }
}

const Table = {
  AccountProcedures: "accountProcedures",
  AccountStorage: "accountStorage",
  AccountVaults: "accountVaults",
  AccountAuth: "accountAuth",
  Accounts: "accounts",
  Transactions: "transactions",
  TransactionScripts: "transactionScripts",
  InputNotes: "inputNotes",
  OutputNotes: "outputNotes",
  NotesScripts: "notesScripts",
  MastForests: "mastForests",
  StateSync: "stateSync",
  BlockHeaders: "blockHeaders",
  PartialBlockchainNodes: "partialBlockchainNodes",
  Tags: "tags",
  ForeignAccountCode: "foreignAccountCode",
};

const db = new Dexie(DATABASE_NAME);
db.version(1).stores({
  [Table.AccountProcedures]: indexes("mastForestRoot", "procedureRoot"),
  [Table.AccountStorage]: indexes("root"),
  [Table.AccountVaults]: indexes("root"),
  [Table.AccountAuth]: indexes("pubKey"),
  [Table.Accounts]: indexes(
    "&accountCommitment",
    "id",
    "codeRoot",
    "storageRoot",
    "vaultRoot"
  ),
  [Table.Transactions]: indexes("id"),
  [Table.TransactionScripts]: indexes("scriptRoot"),
  [Table.InputNotes]: indexes("noteId", "nullifier", "stateDiscriminant"),
  [Table.OutputNotes]: indexes(
    "noteId",
    "recipientDigest",
    "stateDiscriminant",
    "nullifier"
  ),
  [Table.NotesScripts]: indexes("scriptRoot"),
  [Table.MastForests]: indexes("root"),
  [Table.StateSync]: indexes("id"),
  [Table.BlockHeaders]: indexes("blockNum", "hasClientNotes"),
  [Table.PartialBlockchainNodes]: indexes("id"),
  [Table.Tags]: indexes("id++", "tag", "source_note_id", "source_account_id"),
  [Table.ForeignAccountCode]: indexes("accountId"),
});

function indexes(...items) {
  return items.join(",");
}

db.on("populate", () => {
  // Populate the stateSync table with default values
  db.stateSync.put({ id: 1, blockNum: "0" });
});

const accountProcedures = db.table(Table.AccountProcedures);
const accountStorages = db.table(Table.AccountStorage);
const accountVaults = db.table(Table.AccountVaults);
const accountAuths = db.table(Table.AccountAuth);
const accounts = db.table(Table.Accounts);
const transactions = db.table(Table.Transactions);
const transactionScripts = db.table(Table.TransactionScripts);
const inputNotes = db.table(Table.InputNotes);
const outputNotes = db.table(Table.OutputNotes);
const notesScripts = db.table(Table.NotesScripts);
const mastForests = db.table(Table.MastForests);
const stateSync = db.table(Table.StateSync);
const blockHeaders = db.table(Table.BlockHeaders);
const partialBlockchainNodes = db.table(Table.PartialBlockchainNodes);
const tags = db.table(Table.Tags);
const foreignAccountCode = db.table(Table.ForeignAccountCode);

export {
  db,
  accountProcedures,
  accountStorages,
  accountVaults,
  accountAuths,
  accounts,
  transactions,
  transactionScripts,
  inputNotes,
  outputNotes,
  notesScripts,
  mastForests,
  stateSync,
  blockHeaders,
  partialBlockchainNodes,
  tags,
  foreignAccountCode,
};
