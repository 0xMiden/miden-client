// @ts-nocheck
import { test, expect } from "./test-setup";

// AuthScheme enum: 2 = AuthRpoFalcon512 (Falcon), 1 = AuthEcdsaK256Keccak (ECDSA)
const SCHEMES = [
  ["rpoFalconWithRNG", 2],
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
      sdk,
    }) => {
      const secretKey = sdk.AuthSecretKey[secretKeyFn]();
      const commitment = secretKey.publicKey().toCommitment();

      const fromSecret =
        sdk.AccountComponent.createAuthComponentFromSecretKey(secretKey);
      const fromCommitment =
        sdk.AccountComponent.createAuthComponentFromCommitment(
          commitment,
          authSchemeValue
        );

      expect(JSON.stringify(proceduresFromComponent(fromSecret))).toEqual(
        JSON.stringify(proceduresFromComponent(fromCommitment))
      );
    });
  });
});
