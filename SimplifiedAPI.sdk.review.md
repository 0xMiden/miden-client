# SDK Design Review: SimplifiedAPI.md

## 1. Developer Ergonomics — "Time to First Success"

**[PATTERN]** The two-step "build request then submit" pattern adds friction to every transaction. Compare:

```typescript
// This plan — 2 calls for every transaction
const tx = client.mint({ target: "0x...", faucet: "0xFAUCET", amount: 100 });
await client.submitTransaction("0xFAUCET", tx);

// Stripe — 1 call
const charge = await stripe.charges.create({ amount: 100, currency: 'usd', source: token });

// Ethers.js v6 — 1 call
const tx = await signer.sendTransaction({ to: "0x...", value: parseEther("1.0") });

// Viem — 1 call
const hash = await walletClient.sendTransaction({ to: "0x...", value: parseEther("1.0") });
```

Every best-in-class SDK makes the most common operation a single call. The `transfer()` method does this for sends, but `mint()` and `swap()` don't have equivalents. A developer's first experience will be: "I called `mint()` and nothing happened" — because they didn't know to call `submitTransaction()` afterward. That's the opposite of the pit of success.

**[ISSUE]** The `accountId` parameter in `submitTransaction()` is redundant with information already in the transaction request. `mint({ faucet: "0xFAUCET" })` already knows the executing account is the faucet. `send({ from: "0xALICE" })` already knows. Requiring the user to pass it again creates a class of bugs where the two don't match.

```typescript
// Bug: faucet in options doesn't match accountId in submit
const tx = client.mint({ target: "0x...", faucet: "0xFAUCET_A", amount: 100 });
await client.submitTransaction("0xFAUCET_B", tx);  // whoops, wrong faucet
```

Stripe, Viem, and Ethers.js all avoid this by making the account/signer context implicit or embedded in the request.

**[SUGGESTION]** `createP2IDNote()` is documented as a standalone function (Section 2) but the plan never shows how it flows into a transaction. A developer sees: "I created a note... now what?" The gap between `createP2IDNote()` → `TransactionRequestBuilder` → `submitTransaction()` is undocumented. If `send()` handles the common case, clarify when a developer would reach for `createP2IDNote()` directly, and show the complete flow.

## 2. Naming & Conventions

**[NAMING]** `mint()` uses `target` but `send()` uses `to` for the same concept — the recipient of tokens. This will confuse Copilot, trip up developers, and fail the "learn one, predict the rest" test.

```typescript
client.mint({ target: "0xBOB", amount: 100 });   // "target"
client.send({ to: "0xBOB", amount: 100 });        // "to"
```

Pick one. Viem uses `to` everywhere. Ethers.js uses `to` everywhere. `to` is the industry standard for "where tokens go."

**[NAMING]** `listTags()` breaks the `get*` convention used by every other query method:

```typescript
client.getAccounts()       // "get"
client.getTransactions()   // "get"
client.getInputNotes()     // "get"
client.getBalance()        // "get"
client.listTags()          // "list" — why different?
```

Stripe uses `list` everywhere. Supabase uses `select`. This SDK uses `get` — so `listTags` should be `getTags`. Either commit to one verb or the other, but don't mix.

**[NAMING]** `send()` vs `transfer()` — these are near-synonyms in English but do very different things in this SDK:
- `send()` → creates a TransactionRequest (low-level, sync-ish)
- `transfer()` → executes the full flow including submission and optional wait (high-level)

A developer searching for "how to send tokens" will find both and won't know which to use. Compare to Ethers.js where `sendTransaction` always means "submit to the network." Consider renaming `send()` to `buildSendRequest()` or `createSendRequest()` to make the distinction obvious, or better yet, make `send()` do the full flow and provide a `buildSend()` for the low-level case.

**[NAMING]** `fetchPrivateNotes()` uses `fetch` while all other data retrieval uses `get`. The semantic distinction (fetch = from network, get = from local store) is meaningful but invisible to a new developer. Consider `syncPrivateNotes()` since it's really a sync-like operation, or document the naming convention explicitly.

**[SUGGESTION]** `getInputNotes` / `getOutputNotes` use Miden-specific terminology that won't mean anything to a developer new to the protocol. "Input note" and "output note" are UTXO-model terms. Consider aliasing or documenting: `getReceivedNotes()` / `getSentNotes()` or at minimum adding a JSDoc that says "Input notes = notes you can consume (spend). Output notes = notes you've created (sent)."

## 3. Type Design & TypeScript Experience

**[ISSUE]** `ImportAccountOptions` is a "bag of optional fields" that represents three mutually exclusive use cases. TypeScript can enforce this — it should:

```typescript
// Current — allows invalid states
interface ImportAccountOptions {
  id?: string;
  file?: AccountFile;
  seed?: Uint8Array;
  mutable?: boolean;
  auth?: "falcon" | "ecdsa";
}
// What does { id: "0x...", seed: new Uint8Array(32) } mean? Both are set.
// What does { mutable: true } mean? No id, file, or seed — import what?

// Better — discriminated union, invalid states unrepresentable
type ImportAccountInput =
  | string                                          // bech32/hex ID
  | { file: AccountFile }                           // from file
  | { seed: Uint8Array; mutable: boolean; auth: "falcon" | "ecdsa" }  // from seed
```

This is Viem's approach — strict types that guide the developer through IntelliSense. `{ file: ... }` triggers autocomplete for the file variant only.

**[ISSUE]** `NoteQueryOptions` allows passing both `status` and `ids` simultaneously — an ambiguous state:

```typescript
// What does this do? Filter by status AND ids? Or is one ignored?
client.getInputNotes({ status: "committed", ids: ["0x..."] });
```

Should be a discriminated union:
```typescript
type NoteQueryOptions =
  | { status: "consumed" | "committed" | ... }
  | { ids: string[] }
```

**[PATTERN]** Amounts use `number | bigint` for writes but `getBalance()` returns `bigint` for reads. This asymmetry means developers write `amount: 100` but then must compare against `BigInt(100)`:

```typescript
const balance = await client.getBalance("0x...", "0x...");  // returns bigint
if (balance > 100) { /* Type error: can't compare bigint with number */ }
if (balance > 100n) { /* works, but devs won't think of this */ }
```

Viem chose `bigint` everywhere and it's become the de facto standard for web3 TypeScript. Ethers.js v6 made the same choice. Accepting `number` as input sugar is fine but document clearly: "All amounts are `bigint`. Numeric literals are auto-converted."

**[SUGGESTION]** `"falcon" | "ecdsa"` as string literals for auth scheme are cryptic to developers who aren't cryptographers. Consider using a const object for discoverability:

```typescript
// String literals — must know the options exist
auth: "falcon"  // developer: "what are my options?"

// Const object — IntelliSense shows all options
import { AuthScheme } from '@miden-sdk/miden-sdk';
auth: AuthScheme.Falcon  // autocomplete reveals: Falcon, ECDSA
```

This is how Viem handles chain selection (`import { mainnet } from 'viem/chains'`) and AWS SDK v3 handles regions.

## 4. Progressive Disclosure

**[PATTERN]** The plan achieves excellent progressive disclosure for the happy path:

```typescript
// Level 0: Just connect
const client = await WebClient.create();

// Level 1: Create an account
const wallet = await client.createWallet();

// Level 2: Send tokens
await client.transfer({ from: "0x...", to: "0x...", faucet: "0x...", amount: 100 });
```

This is comparable to Supabase's onboarding:
```typescript
const supabase = createClient(url, key);
const { data } = await supabase.from('users').select();
```

**[ISSUE]** However, there's a cliff between "use `transfer()`" and "build custom transactions." The plan doesn't show any intermediate step. A developer who needs to do anything beyond mint/send/consume/swap must jump directly to:

```typescript
const builder = new TransactionRequestBuilder()
  .withOwnOutputNotes(new OutputNoteArray([outputNote]))
  .withUnauthenticatedInputNotes(new NoteAndArgsArray([...]))
  .withCustomScript(script)
  .build();
```

This is the un-simplified current API with typed array wrappers. The plan should either simplify `TransactionRequestBuilder` too, or explicitly mark it as the "power user escape hatch" and document the boundary.

## 5. Consistency & Predictability

**[ISSUE]** Return type inconsistency across transaction builders:

```typescript
mint(options: MintOptions): TransactionRequest;                    // sync
send(options: SendOptions): TransactionRequest;                    // sync
consume(notes: NoteInput | NoteInput[]): Promise<TransactionRequest>;  // ASYNC
swap(options: SwapOptions): TransactionRequest;                    // sync
```

Three are sync, one is async. A developer who learns `const tx = client.mint(...)` will write `const tx = client.consume(...)` and get a Promise instead of a TransactionRequest. This is the kind of inconsistency that causes subtle bugs caught only at runtime.

Options:
1. Make them all async (even if some resolve immediately) — consistent, follows Supabase pattern where all queries are async
2. Document prominently why `consume()` differs (it resolves string note IDs from the store)

Option 1 is strongly recommended. `async` has near-zero overhead for sync-ready Promises, and consistency is worth more than micro-optimization.

**[ISSUE]** The "executing account" is expressed differently across operations:

| Method | Executing account field | Also in submitTransaction? |
|--------|------------------------|---------------------------|
| `mint()` | `faucet` (implicit) | Yes, as `accountId` |
| `send()` | `from` | Yes, as `accountId` |
| `consume()` | Not specified at all | Yes, as `accountId` |
| `swap()` | `from` | Yes, as `accountId` |
| `transfer()` | `from` | N/A (internal) |
| `consumeAllNotes()` | positional `accountId` | N/A (internal) |
| `mintAndConsume()` | `faucet` (implicit) + `target` | N/A (internal) |

This is three different patterns for expressing "which account runs this." Stripe and Viem both solve this by establishing the account context once (Stripe: API key, Viem: `WalletClient` with account).

**[SUGGESTION]** `consume()` has no account context at all in the request-building phase — the account only appears in `submitTransaction()`. But `consumeAllNotes()` takes `accountId` as its first positional arg (not in an options object). These two related APIs have completely different signatures for the same concept.

## 6. Modern SDK Patterns

**[PATTERN]** The `WebClient` class combines reads, writes, account management, sync, import/export, and lifecycle into a single ~30-method God Object. Compare:

```typescript
// This SDK — one class does everything
client.create()
client.createWallet()
client.send()
client.getBalance()
client.syncState()
client.importAccount()
client.addTag()
client.fetchPrivateNotes()

// Viem — separated by concern
const publicClient = createPublicClient({ ... });    // reads
const walletClient = createWalletClient({ ... });    // writes
// Each has ~10 methods, not ~30

// Stripe — resource-based
stripe.charges.create()
stripe.customers.retrieve()
stripe.subscriptions.list()
// Namespaced, discoverable
```

This doesn't mean the SDK must split into multiple classes, but consider namespace grouping:

```typescript
client.accounts.create()        // createWallet, createFaucet
client.accounts.get()           // getAccount, getAccountDetails
client.transactions.mint()      // mint, send, consume, swap
client.transactions.submit()    // submitTransaction
client.notes.getInput()         // getInputNotes
client.notes.getOutput()        // getOutputNotes
```

This improves IntelliSense discoverability — type `client.` and see 5 namespaces instead of 30 methods.

**[SUGGESTION]** No event/observable pattern exists. Transaction confirmation uses polling (`waitForTransaction`) but doesn't expose events. Modern SDKs provide:

```typescript
// Ethers.js
provider.on('block', callback);

// Supabase Realtime
supabase.channel('changes').on('postgres_changes', callback).subscribe();

// Potential Miden pattern
client.on('sync', (summary) => { /* new block processed */ });
client.on('noteReceived', (note) => { /* new note for tracked accounts */ });
```

Even if not implemented now, the plan should acknowledge this as a future direction.

**[PATTERN]** `createP2IDNote` and `createP2IDENote` as standalone functions is the correct Viem-style tree-shakeable pattern. However, every other method is on the class. If the goal is tree-shakeability, consider making the transaction builders standalone too:

```typescript
// Current plan — mixed patterns
import { createP2IDNote } from '@miden-sdk/miden-sdk';  // standalone
client.mint({ ... });  // class method

// Viem-consistent — all standalone
import { mint, send, consume } from '@miden-sdk/miden-sdk';
const tx = mint(client, { target: "0x...", faucet: "0x...", amount: 100 });
```

This is a bigger architectural decision and may not be worth the migration cost, but it's worth noting the inconsistency.

## 7. Comparison to Web3 SDK Peers

**This SDK most closely resembles Ethers.js v5** — class-based God Object, factory constructor, string IDs, method-per-operation.

**It should aspire toward Starknet.js** — which has the closest domain model (account abstraction, STARK-based proofs, note-like UTXOs in its newer patterns). Key lessons from Starknet.js:

```typescript
// Starknet.js — Account is the primary actor
const account = new Account(provider, address, privateKey);
const result = await account.execute(call);

// Potential Miden equivalent
const account = client.account("0xALICE");
await account.send({ to: "0xBOB", faucet: "0x...", amount: 100 });
await account.mint({ target: "0xBOB", amount: 100 });  // only works for faucet accounts
const balance = await account.getBalance("0xFAUCET");
```

This would solve the "accountId appears twice" problem and make the SDK more intuitive — operations are scoped to an account, which is how users think about blockchain interactions.

**Viem patterns worth adopting:**
1. **Hex branded types**: `type Address = \`0x${string}\`` — catches wrong-format strings at compile time
2. **Action separation**: Reads vs writes in different clients — prevents accidental mutations
3. **Explicit over magic**: No auto-conversion that hides what's happening

**Ethers.js v6 lessons (what to avoid):**
1. Ethers.js v6's migration from v5 was painful because of BigNumber→bigint changes. If this SDK launches with `number | bigint`, it will face a similar migration later. Better to go `bigint`-only from the start.

## 8. Edge Cases & Real-World Usage

**[ISSUE]** `mintAndConsume()` is a multi-step composed operation (mint → sync → consume) with no documented partial-failure semantics. If mint succeeds but sync times out:
- Are the minted tokens lost? (No — they exist on-chain, but the client doesn't know about them)
- Can the user retry `mintAndConsume()`? (Will it mint again? That's a double-mint)
- How does the user recover? (Manually sync, then consume?)

This is a real production concern. Firebase handles this with idempotency keys. Stripe uses them too. The plan should specify:
1. Is `mintAndConsume()` idempotent?
2. What error does a partial failure throw?
3. How does the developer recover?

**[QUESTION]** `waitForTransaction()` polls via `syncState()`. What's the polling interval? Is it configurable? Aggressive polling wastes bandwidth; slow polling frustrates users. Ethers.js uses provider-specific intervals (e.g., 4 seconds for Ethereum). What's appropriate for Miden's block time?

**[QUESTION]** Can multiple `WebClient` instances coexist? The `storeName` option suggests yes, but the Web Worker architecture suggests they might share a worker. If two clients call `syncState()` concurrently, do Web Locks handle this correctly? The current `index.js` uses `navigator.locks.request` for sync — does this extend to the simplified APIs?

**[SUGGESTION]** No logging or debug support is mentioned. When a composed operation like `transfer()` fails in production, how does the developer debug it? Consider:
```typescript
const client = await WebClient.create({
  debug: true,  // logs all WASM calls and their durations
  // or
  logger: customLogger  // à la AWS SDK v3's middleware logging
});
```

**[SUGGESTION]** The plan mentions `MockWebClient` exists but doesn't specify how simplified APIs interact with it. Will `MockWebClient.create()` also work? Will `mockClient.transfer()` go through the mock chain? This matters for testing — if simplified APIs bypass the mock layer, they're untestable.

---

## SDK Scorecard

| Dimension | Score (1-5) | Notes |
|-----------|:-----------:|-------|
| **Discoverability** | 3 | 30 methods on one class hurts IntelliSense; mixed naming (`get`/`list`/`fetch`) adds confusion. Strong improvement over current API but not best-in-class. |
| **Learnability** | 4 | Excellent progressive disclosure for happy path (create→wallet→send). The cliff to custom transactions is steep. Good defaults throughout. |
| **Consistency** | 2 | `target` vs `to`, `get` vs `list` vs `fetch`, sync vs async return types, three patterns for "executing account." This is the weakest dimension. |
| **TypeScript experience** | 3 | Options objects are good; `number | bigint` is pragmatic. But `ImportAccountOptions` allows invalid states, query options are ambiguous, and there are no branded types for hex/bech32 strings. |
| **Composability** | 3 | Low-level primitives exist (`createP2IDNote`, `TransactionRequestBuilder`) but the bridge between simplified and advanced APIs is undocumented. Standalone functions for notes is good. |

## Top 3 Changes

1. **Unify the "executing account" pattern.** Either embed the account in the transaction request so `submitTransaction` doesn't need a redundant `accountId` param, or adopt an account-scoped pattern (`client.account("0x...").send(...)`) à la Starknet.js. This eliminates the biggest class of developer errors and removes the `target`/`from`/`faucet`-as-implicit-executor ambiguity. This single change would fix 4 of the findings above.

2. **Make all transaction builders async and use consistent field names.** `to` everywhere (not `target`), all return `Promise<TransactionRequest>`, `getTags()` not `listTags()`. Consistency is the #1 predictor of SDK learnability — once a developer gets one thing right, everything else should follow the same pattern.

3. **Use discriminated unions for multi-variant options.** `ImportAccountOptions`, `NoteQueryOptions`, and `TransactionQueryOptions` should all use TypeScript discriminated unions so invalid states are unrepresentable. This turns runtime errors into compile-time errors and makes IntelliSense dramatically more helpful.

## Closest Peer SDK

**Currently resembles:** Ethers.js v5 — class-based God Object, factory constructor, string IDs, method-per-operation.

**Should aspire to:** A hybrid of **Starknet.js** (account-centric operations, proof-based architecture) and **Viem** (strict TypeScript, consistent patterns, standalone functions where appropriate). Starknet.js provides the right mental model for a ZK-rollup SDK; Viem provides the right TypeScript ergonomics.
