import test from "./playwright.global.setup";
import { expect } from "@playwright/test";

test.describe("signature", () => {
  test("should produce a valid signature", async ({ page }) => {
    const isValid = await page.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const message = new window.Word(
        new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
      );
      const signature = secretKey.sign(message);
      const isValid = secretKey.publicKey().verify(message, signature);

      return isValid;
    });
    expect(isValid).toEqual(true);
  });

  test("should not verify the wrong message", async ({ page }) => {
    const isValid = await page.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const message = new window.Word(
        new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
      );
      const wrongMessage = new window.Word(
        new BigUint64Array([BigInt(5), BigInt(6), BigInt(7), BigInt(8)])
      );
      const signature = secretKey.sign(message);
      const isValid = secretKey.publicKey().verify(wrongMessage, signature);

      return isValid;
    });
    expect(isValid).toEqual(false);
  });

  test("should not verify the signature of a different key", async ({
    page,
  }) => {
    const isValid = await page.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const message = new window.Word(
        new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
      );
      const signature = secretKey.sign(message);
      const differentSecretKey = window.SecretKey.withRng();
      const isValid = differentSecretKey.publicKey().verify(message, signature);

      return isValid;
    });
    expect(isValid).toEqual(false);
  });

  test("should be able to serialize and deserialize a signature", async ({
    page,
  }) => {
    const isValid = await page.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const message = new window.Word(
        new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
      );
      const signature = secretKey.sign(message);
      const serializedSignature = signature.serialize();
      const deserializedSignature =
        window.Signature.deserialize(serializedSignature);

      const isValid = secretKey
        .publicKey()
        .verify(message, deserializedSignature);

      return isValid;
    });
    expect(isValid).toEqual(true);
  });
});

test.describe("public key", () => {
  test("should be able to serialize and deserialize a public key", async ({
    page,
  }) => {
    const isValid = await page.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const publicKey = secretKey.publicKey();
      const serializedPublicKey = publicKey.serialize();
      const deserializedPublicKey =
        window.PublicKey.deserialize(serializedPublicKey);
      const serializedDeserializedPublicKey = deserializedPublicKey.serialize();
      return (
        serializedPublicKey.toString() ===
        serializedDeserializedPublicKey.toString()
      );
    });
    expect(isValid).toEqual(true);
  });
});

test.describe("secret key", () => {
  test("should be able to serialize and deserialize a secret key", async ({
    page,
  }) => {
    const isValid = await page.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const serializedSecretKey = secretKey.serialize();
      const deserializedSecretKey =
        window.SecretKey.deserialize(serializedSecretKey);
      const serializedDeserializedSecretKey = deserializedSecretKey.serialize();
      return (
        serializedSecretKey.toString() ===
        serializedDeserializedSecretKey.toString()
      );
    });
    expect(isValid).toEqual(true);
  });
});
