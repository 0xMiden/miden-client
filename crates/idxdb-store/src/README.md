## Formatting for .js files
Please install the VSCode prettier extension and set as default formatter for js 

## Javascript & Typescript interop

Besides from compiling Rust to WebAssembly, we sometimes have the need to call some external
Javascript function. This is not bad per se, but it has a problem: Javascript is a
dynamic, weakly typed language, any type guarantees we get from Rust are erased once we start 
calling JS code. To mitigate that, we've started to incorporate Typescript where we once used 
Javascript. The setup consists of .ts files under the `src/ts` that get compiled to .js 
files under `src/js`. This is because to use extern functions, we still need to import raw
.js files.

To unify and make this setup straightforward, the top-most makefile from this project has a
useful target: `make rust-client-ts-build`, which takes the .ts files and compiles them down to .js files.

## Testing

Tests use [vitest](https://vitest.dev/) with [fake-indexeddb](https://github.com/dumbmatter/fakeIndexedDB) to run IndexedDB operations in Node.js.

```bash
yarn test    # run tests
yarn build   # compile TS → JS
yarn lint    # build + eslint
```

## IndexedDB schema migrations

The schema is defined in `ts/schema.ts` using [Dexie.js](https://dexie.org/) version blocks. See the comment above `version(1)` in the `MidenDatabase` constructor for how to add migrations. Migration tests go in `ts/schema.test.ts`.
