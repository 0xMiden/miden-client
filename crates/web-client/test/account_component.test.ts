// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("account component auth constructors", () => {
  test("createAuthComponentFromCommitment matches secret-key variant (Falcon, authScheme=2)", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.rpoFalconWithRNG();
      const commitment = secretKey.publicKey().toCommitment();

      const fromSecret =
        sdk.AccountComponent.createAuthComponentFromSecretKey(secretKey);
      const fromCommitment =
        sdk.AccountComponent.createAuthComponentFromCommitment(commitment, 2);

      const proceduresFromComponent = (component) =>
        component
          .getProcedures()
          .map((procedure) => procedure.digest.toHex())
          .sort();

      return {
        fromSecretProcs: JSON.stringify(proceduresFromComponent(fromSecret)),
        fromCommitmentProcs: JSON.stringify(
          proceduresFromComponent(fromCommitment)
        ),
      };
    });
    expect(result.fromSecretProcs).toEqual(result.fromCommitmentProcs);
  });

  test("createAuthComponentFromCommitment matches secret-key variant (ECDSA, authScheme=1)", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.ecdsaWithRNG();
      const commitment = secretKey.publicKey().toCommitment();

      const fromSecret =
        sdk.AccountComponent.createAuthComponentFromSecretKey(secretKey);
      const fromCommitment =
        sdk.AccountComponent.createAuthComponentFromCommitment(commitment, 1);

      const proceduresFromComponent = (component) =>
        component
          .getProcedures()
          .map((procedure) => procedure.digest.toHex())
          .sort();

      return {
        fromSecretProcs: JSON.stringify(proceduresFromComponent(fromSecret)),
        fromCommitmentProcs: JSON.stringify(
          proceduresFromComponent(fromCommitment)
        ),
      };
    });
    expect(result.fromSecretProcs).toEqual(result.fromCommitmentProcs);
  });
});
