# Testing

The .wasm must be run within the context of a webpage. To this end, we've set up a Mocha
test suite which hosts the .wasm on a local server and then executes WebClient commands
within the context of the web page.

## Prerequisites 

1. [Node](https://nodejs.org/en/download/package-manager)
  - Node Version >= v20.16.0
1. These instructions utilize [yarn](https://classic.yarnpkg.com/lang/en/docs/install) but can also be executed with npm

## Running tests

1. Install dependencies via `yarn`
2. Ensure the .wasm is built by running `yarn build` (use `yarn build-dev` for a shorter build time)
3. In crates/web-client run `yarn test` to run all tests

   - Can alternatively run `yarn test:clean` to run the .wasm build process prior to testing. We provide both paths as the build process can take some time.

4. To run an individual test by name run `yarn test test/name_of_test_file.ts`.

## Writing tests

1. The test setup in `playwright.global.setup` exposes the `WebClient` class (under
the global `Window` object) which provides a static method `createClient` to create an instance of the web client.
   - Any further setup of wasm code should be done in this file and similarly expose a function for testing here
2. `webClientTestUtils.ts` should contain all interfaces for interacting with the web client. If further methods need to be added, follow existing patterns which use the exposed `page` and pass through any required arguments to the page execution. Example:

```
/**
 *
 * @param {string} arg1
 * @param {boolean} arg2
 *
 * @returns {Promise<string>} The result
 */
export const webClientCall = async (arg1, arg2) => {
  return await testingPage.evaluate(
    async ({arg1, arg2}) => {
      /** @type {WebClient} */
      // window.client is defined in the setup under
      // playwright.global.setup.ts
      const client = window.client;
      const result = client.webClientCall(_arg1, _arg2);

      return result;
    },
    // Careful! Multiple arguments require to be wrapped inside an object.
    { arg1, arg2 }
  );
};
```

- Add JSDocs to methods. This will allow typing in the `*.test.ts` files.
- Similarly, the boilerplate for passing args through as shown above is necessary due to scoping and how the testing framework works.

## Debugging

1. When inside of a `page.evaluate` , console logs are being sent to the servers console rather than your IDE's. You can uncomment the line as seen below in the `playwright.global.setup`:

```
    page.on("console", (msg) => console.log("PAGE LOG:", msg.text()));
```

This will forward logs from the server to your terminal logs

## Troubleshooting

1. When trying to run the tests, if you receive an error about missing browsers,
   install them with: `yarn playwright install` and then run the tests again.
