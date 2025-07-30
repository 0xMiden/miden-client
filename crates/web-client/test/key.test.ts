import { expect } from "chai";
import { testingPage } from "./mocha.global.setup.mjs";

describe("signature", () => {
  it("should produce a valid signature", async () => {
    const isValid = await testingPage.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const message = new window.Word(
        new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
      );
      const signature = secretKey.sign(message);
      const isValid = secretKey.publicKey().verify(message, signature);

      return isValid;
    });
    expect(isValid).to.be.true;
  });

  it("should not verify the wrong message", async () => {
    const isValid = await testingPage.evaluate(() => {
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
    expect(isValid).to.be.false;
  });

  it("should not verify the signature of a different key", async () => {
    const isValid = await testingPage.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const message = new window.Word(
        new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
      );
      const signature = secretKey.sign(message);
      const differentSecretKey = window.SecretKey.withRng();
      const isValid = differentSecretKey.publicKey().verify(message, signature);

      return isValid;
    });
    expect(isValid).to.be.false;
  });

  it("should be able to serialize and deserialize a signature", async () => {
    const isValid = await testingPage.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const message = new window.Word(
        new BigUint64Array([BigInt(1), BigInt(2), BigInt(3), BigInt(4)])
      );
      const signature = secretKey.sign(message);
      const serializedSignature = signature.toHex();
      const deserializedSignature =
        window.Signature.fromHex(serializedSignature);

      const isValid = secretKey
        .publicKey()
        .verify(message, deserializedSignature);

      return isValid;
    });
    expect(isValid).to.be.true;
  });
});

describe("public key", () => {
  it("should be able to serialize and deserialize a public key", async () => {
    const isValid = await testingPage.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const publicKey = secretKey.publicKey();
      const serializedPublicKey = publicKey.toHex();
      const deserializedPublicKey =
        window.PublicKey.fromHex(serializedPublicKey);
      const serializedDeserializedPublicKey = deserializedPublicKey.toHex();
      return serializedPublicKey === serializedDeserializedPublicKey;
    });
    expect(isValid).to.be.true;
  });
});

describe("secret key", () => {
  it("should be able to serialize and deserialize a secret key", async () => {
    const isValid = await testingPage.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const serializedSecretKey = secretKey.toHex();
      const deserializedSecretKey =
        window.SecretKey.fromHex(serializedSecretKey);
      const serializedDeserializedSecretKey = deserializedSecretKey.toHex();
      return serializedSecretKey === serializedDeserializedSecretKey;
    });
    expect(isValid).to.be.true;
  });
});
