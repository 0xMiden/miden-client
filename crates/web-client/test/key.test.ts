// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("signature", () => {
  test("should produce a valid signature: Falcon Scheme", async ({ run }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.rpoFalconWithRNG();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signature = secretKey.sign(message);
      return secretKey.publicKey().verify(message, signature);
    });
    expect(result).toEqual(true);
  });

  test("should produce a valid signature: ECDSA Scheme", async ({ run }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.ecdsaWithRNG();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signature = secretKey.sign(message);
      return secretKey.publicKey().verify(message, signature);
    });
    expect(result).toEqual(true);
  });

  test("should not verify the wrong message: Falcon Scheme", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.rpoFalconWithRNG();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const wrongMessage = new sdk.Word(sdk.u64Array([5, 6, 7, 8]));
      const signature = secretKey.sign(message);
      return secretKey.publicKey().verify(wrongMessage, signature);
    });
    expect(result).toEqual(false);
  });

  test("should not verify the wrong message: ECDSA Scheme", async ({ run }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.ecdsaWithRNG();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const wrongMessage = new sdk.Word(sdk.u64Array([5, 6, 7, 8]));
      const signature = secretKey.sign(message);
      return secretKey.publicKey().verify(wrongMessage, signature);
    });
    expect(result).toEqual(false);
  });

  test("should not verify the signature of a different key: Falcon Scheme", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.rpoFalconWithRNG();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signature = secretKey.sign(message);
      const differentSecretKey = sdk.AuthSecretKey.rpoFalconWithRNG();
      return differentSecretKey.publicKey().verify(message, signature);
    });
    expect(result).toEqual(false);
  });

  test("should not verify the signature of a different key: ECDSA Scheme", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.ecdsaWithRNG();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signature = secretKey.sign(message);
      const differentSecretKey = sdk.AuthSecretKey.ecdsaWithRNG();
      return differentSecretKey.publicKey().verify(message, signature);
    });
    expect(result).toEqual(false);
  });

  test("should be able to serialize and deserialize a signature: Falcon Scheme", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.rpoFalconWithRNG();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signature = secretKey.sign(message);
      const serializedSignature = signature.serialize();
      const deserializedSignature =
        sdk.Signature.deserialize(serializedSignature);
      return secretKey.publicKey().verify(message, deserializedSignature);
    });
    expect(result).toEqual(true);
  });

  test("should be able to serialize and deserialize a signature: ECDSA Scheme", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.ecdsaWithRNG();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signature = secretKey.sign(message);
      const serializedSignature = signature.serialize();
      const deserializedSignature =
        sdk.Signature.deserialize(serializedSignature);
      return secretKey.publicKey().verify(message, deserializedSignature);
    });
    expect(result).toEqual(true);
  });
});

test.describe("public key", () => {
  test("should be able to serialize and deserialize a public key: Falcon Scheme", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.rpoFalconWithRNG();
      const publicKey = secretKey.publicKey();
      const serializedPublicKey = publicKey.serialize();
      const deserializedPublicKey =
        sdk.PublicKey.deserialize(serializedPublicKey);
      const serializedDeserializedPublicKey = deserializedPublicKey.serialize();
      return {
        original: serializedPublicKey.toString(),
        roundtripped: serializedDeserializedPublicKey.toString(),
      };
    });
    expect(result.original).toEqual(result.roundtripped);
  });

  test("should be able to serialize and deserialize a public key: ECDSA Scheme", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.ecdsaWithRNG();
      const publicKey = secretKey.publicKey();
      const serializedPublicKey = publicKey.serialize();
      const deserializedPublicKey =
        sdk.PublicKey.deserialize(serializedPublicKey);
      const serializedDeserializedPublicKey = deserializedPublicKey.serialize();
      return {
        original: serializedPublicKey.toString(),
        roundtripped: serializedDeserializedPublicKey.toString(),
      };
    });
    expect(result.original).toEqual(result.roundtripped);
  });
});

test.describe("signing inputs", () => {
  test("should be able to sign and verify an arbitrary array of felts: Falcon Scheme", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.rpoFalconWithRNG();
      const otherSecretKey = sdk.AuthSecretKey.rpoFalconWithRNG();
      const message = Array.from(
        { length: 128 },
        (_, i) => new sdk.Felt(sdk.u64(i))
      );
      const signingInputs = sdk.SigningInputs.newArbitrary(message);
      const signature = secretKey.signData(signingInputs);
      const isValid = secretKey
        .publicKey()
        .verifyData(signingInputs, signature);
      const isValidOther = otherSecretKey
        .publicKey()
        .verifyData(signingInputs, signature);
      return { isValid, isValidOther };
    });
    expect(result.isValid).toBe(true);
    expect(result.isValidOther).toBe(false);
  });

  test("should be able to sign and verify an arbitrary array of felts: ECDSA Scheme", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.ecdsaWithRNG();
      const otherSecretKey = sdk.AuthSecretKey.ecdsaWithRNG();
      const message = Array.from(
        { length: 128 },
        (_, i) => new sdk.Felt(sdk.u64(i))
      );
      const signingInputs = sdk.SigningInputs.newArbitrary(message);
      const signature = secretKey.signData(signingInputs);
      const isValid = secretKey
        .publicKey()
        .verifyData(signingInputs, signature);
      const isValidOther = otherSecretKey
        .publicKey()
        .verifyData(signingInputs, signature);
      return { isValid, isValidOther };
    });
    expect(result.isValid).toBe(true);
    expect(result.isValidOther).toBe(false);
  });

  test("should be able to sign and verify a blind word: Falcon Scheme", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.rpoFalconWithRNG();
      const otherSecretKey = sdk.AuthSecretKey.rpoFalconWithRNG();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signingInputs = sdk.SigningInputs.newBlind(message);
      const signature = secretKey.signData(signingInputs);
      const isValid = secretKey
        .publicKey()
        .verifyData(signingInputs, signature);
      const isValidOther = otherSecretKey
        .publicKey()
        .verifyData(signingInputs, signature);
      return { isValid, isValidOther };
    });
    expect(result.isValid).toBe(true);
    expect(result.isValidOther).toBe(false);
  });

  test("should be able to sign and verify a blind word: ECDSA Scheme", async ({
    run,
  }) => {
    const result = await run(async ({ sdk }) => {
      const secretKey = sdk.AuthSecretKey.ecdsaWithRNG();
      const otherSecretKey = sdk.AuthSecretKey.ecdsaWithRNG();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signingInputs = sdk.SigningInputs.newBlind(message);
      const signature = secretKey.signData(signingInputs);
      const isValid = secretKey
        .publicKey()
        .verifyData(signingInputs, signature);
      const isValidOther = otherSecretKey
        .publicKey()
        .verifyData(signingInputs, signature);
      return { isValid, isValidOther };
    });
    expect(result.isValid).toBe(true);
    expect(result.isValidOther).toBe(false);
  });
});
