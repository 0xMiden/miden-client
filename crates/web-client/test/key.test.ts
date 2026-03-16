// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("signature", () => {
  [
    ["rpoFalconWithRNG", "Falcon Scheme"],
    ["ecdsaWithRNG", "ECDSA Scheme"],
  ].forEach(([signatureFunction, signatureScheme]) => {
    test(`should produce a valid signature: ${signatureScheme}`, async ({
      sdk,
    }) => {
      const secretKey = sdk.AuthSecretKey[signatureFunction]();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signature = secretKey.sign(message);
      const isValid = secretKey.publicKey().verify(message, signature);

      expect(isValid).toEqual(true);
    });

    test(`should not verify the wrong message: ${signatureScheme}`, async ({
      sdk,
    }) => {
      const secretKey = sdk.AuthSecretKey[signatureFunction]();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const wrongMessage = new sdk.Word(sdk.u64Array([5, 6, 7, 8]));
      const signature = secretKey.sign(message);
      const isValid = secretKey.publicKey().verify(wrongMessage, signature);

      expect(isValid).toEqual(false);
    });

    test(`should not verify the signature of a different key: ${signatureScheme}`, async ({
      sdk,
    }) => {
      const secretKey = sdk.AuthSecretKey[signatureFunction]();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signature = secretKey.sign(message);
      const differentSecretKey = sdk.AuthSecretKey[signatureFunction]();
      const isValid = differentSecretKey.publicKey().verify(message, signature);

      expect(isValid).toEqual(false);
    });

    test(`should be able to serialize and deserialize a signature: ${signatureScheme}`, async ({
      sdk,
    }) => {
      const secretKey = sdk.AuthSecretKey[signatureFunction]();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signature = secretKey.sign(message);
      const serializedSignature = signature.serialize();
      const deserializedSignature =
        sdk.Signature.deserialize(serializedSignature);

      const isValid = secretKey
        .publicKey()
        .verify(message, deserializedSignature);

      expect(isValid).toEqual(true);
    });
  });
});

test.describe("public key", () => {
  [
    ["rpoFalconWithRNG", "Falcon Scheme"],
    ["ecdsaWithRNG", "ECDSA Scheme"],
  ].forEach(([signatureFunction, signatureScheme]) => {
    test(`should be able to serialize and deserialize a public key: ${signatureScheme}`, async ({
      sdk,
    }) => {
      const secretKey = sdk.AuthSecretKey[signatureFunction]();
      const publicKey = secretKey.publicKey();
      const serializedPublicKey = publicKey.serialize();
      const deserializedPublicKey =
        sdk.PublicKey.deserialize(serializedPublicKey);
      const serializedDeserializedPublicKey = deserializedPublicKey.serialize();

      expect(serializedPublicKey.toString()).toEqual(
        serializedDeserializedPublicKey.toString()
      );
    });
  });
});

test.describe("signing inputs", () => {
  [
    ["rpoFalconWithRNG", "Falcon Scheme"],
    ["ecdsaWithRNG", "ECDSA Scheme"],
  ].forEach(([signatureFunction, signatureScheme]) => {
    test(`should be able to sign and verify an arbitrary array of felts: ${signatureScheme}`, async ({
      sdk,
    }) => {
      const secretKey = sdk.AuthSecretKey[signatureFunction]();
      const otherSecretKey = sdk.AuthSecretKey[signatureFunction]();
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

      expect(isValid).toBe(true);
      expect(isValidOther).toBe(false);
    });

    test(`should be able to sign and verify a blind word: ${signatureScheme}`, async ({
      sdk,
    }) => {
      const secretKey = sdk.AuthSecretKey[signatureFunction]();
      const otherSecretKey = sdk.AuthSecretKey[signatureFunction]();
      const message = new sdk.Word(sdk.u64Array([1, 2, 3, 4]));
      const signingInputs = sdk.SigningInputs.newBlind(message);
      const signature = secretKey.signData(signingInputs);
      const isValid = secretKey
        .publicKey()
        .verifyData(signingInputs, signature);
      const isValidOther = otherSecretKey
        .publicKey()
        .verifyData(signingInputs, signature);

      expect(isValid).toBe(true);
      expect(isValidOther).toBe(false);
    });
  });
});
