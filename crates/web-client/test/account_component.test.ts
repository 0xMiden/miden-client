import test from "./playwright.global.setup";
import { expect } from "@playwright/test";

// WASM AuthScheme enum: 0 = AuthRpoFalcon512 (Falcon), 1 = AuthEcdsaK256Keccak (ECDSA)
const SCHEMES = [
  ["rpoFalconWithRNG", 0],
  ["ecdsaWithRNG", 1],
] as const;

const proceduresFromComponent = (component: any) =>
  component
    .getProcedures()
    .map((procedure: any) => procedure.digest.toHex())
    .sort();

test.describe("account component auth constructors", () => {
  SCHEMES.forEach(([secretKeyFn, authSchemeValue]) => {
    test(`createAuthComponentFromCommitment matches secret-key variant (${authSchemeValue})`, async ({
      page,
    }) => {
      const digestsMatch = await page.evaluate(
        ({ _secretKeyFn, _authSchemeValue }) => {
          const secretKey = window.AuthSecretKey[_secretKeyFn]();
          const commitment = secretKey.publicKey().toCommitment();

          const fromSecret =
            window.AccountComponent.createAuthComponentFromSecretKey(secretKey);
          const fromCommitment =
            window.AccountComponent.createAuthComponentFromCommitment(
              commitment,
              _authSchemeValue
            );

          const toHexList = (component: any) =>
            component
              .getProcedures()
              .map((procedure: any) => procedure.digest.toHex())
              .sort();

          return (
            JSON.stringify(toHexList(fromSecret)) ===
            JSON.stringify(toHexList(fromCommitment))
          );
        },
        { _secretKeyFn: secretKeyFn, _authSchemeValue: authSchemeValue }
      );

      expect(digestsMatch).toBe(true);
    });
  });
});
