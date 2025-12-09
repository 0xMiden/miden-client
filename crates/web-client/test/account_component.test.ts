import test from "./playwright.global.setup";
import { expect } from "@playwright/test";

const SCHEMES = [
  ["rpoFalconWithRNG", "AuthRpoFalcon512"],
  ["ecdsaWithRNG", "AuthEcdsaK256Keccak"],
] as const;

const proceduresFromComponent = (component: any) =>
  component
    .getProcedures()
    .map((procedure: any) => procedure.digest().toHex())
    .sort();

test.describe("account component auth constructors", () => {
  SCHEMES.forEach(([secretKeyFn, authSchemeKey]) => {
    test(`createAuthComponentFromCommitment matches secret-key variant (${authSchemeKey})`, async ({
      page,
    }) => {
      const digestsMatch = await page.evaluate(
        ({ _secretKeyFn, _authSchemeKey }) => {
          const secretKey = window.SecretKey[_secretKeyFn]();
          const commitment = secretKey.publicKey().toCommitment();

          const fromSecret =
            window.AccountComponent.createAuthComponentFromSecretKey(secretKey);
          const fromCommitment =
            window.AccountComponent.createAuthComponentFromCommitment(
              commitment,
              window.AuthScheme[_authSchemeKey]
            );

          const toHexList = (component: any) =>
            component
              .getProcedures()
              .map((procedure: any) => procedure.digest().toHex())
              .sort();

          return (
            JSON.stringify(toHexList(fromSecret)) ===
            JSON.stringify(toHexList(fromCommitment))
          );
        },
        { _secretKeyFn: secretKeyFn, _authSchemeKey: authSchemeKey }
      );

      expect(digestsMatch).toBe(true);
    });
  });
});
