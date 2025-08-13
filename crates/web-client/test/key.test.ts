import { expect } from "chai";
import { testingPage } from "./mocha.global.setup.mjs";

describe("signature", () => {
  it("should produce a valid signature", async () => {
    const isValid = await testingPage.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const message = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
      const signature = secretKey.sign(message);
      const isValid = secretKey.publicKey().verify(message, signature);

      return isValid;
    });
    expect(isValid).to.be.true;
  });

  it("should not verify the wrong message", async () => {
    const isValid = await testingPage.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const message = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
      const wrongMessage = new window.Word(
        new BigUint64Array([5n, 6n, 7n, 8n])
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
      const message = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
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
      const message = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
      const signature = secretKey.sign(message);
      const serializedSignature = signature.serialize();
      const deserializedSignature =
        window.Signature.deserialize(serializedSignature);

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
      const serializedPublicKey = publicKey.serialize();
      const deserializedPublicKey =
        window.PublicKey.deserialize(serializedPublicKey);
      const serializedDeserializedPublicKey = deserializedPublicKey.serialize();
      return (
        serializedPublicKey.toString() ===
        serializedDeserializedPublicKey.toString()
      );
    });
    expect(isValid).to.be.true;
  });
});

describe("secret key", () => {
  it("should be able to serialize and deserialize a secret key", async () => {
    const isValid = await testingPage.evaluate(() => {
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
    expect(isValid).to.be.true;
  });
});

describe.only("signing inputs", () => {
  it("should be able to sign and verify an arbitrary array of felts", async () => {
    const { isValid, isValidOther } = await testingPage.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const otherSecretKey = window.SecretKey.withRng();
      const message = [
        new window.Felt(1n),
        new window.Felt(2n),
        new window.Felt(3n),
        new window.Felt(4n),
      ];
      const signingInputs = window.SigningInputs.newArbitrary(message);
      const signature = signingInputs.sign(secretKey);
      const isValid = signingInputs.verify(secretKey.publicKey(), signature);
      const isValidOther = signingInputs.verify(
        otherSecretKey.publicKey(),
        signature
      );

      return { isValid, isValidOther };
    });
    expect(isValid).to.be.true;
    expect(isValidOther).to.be.false;
  });

  it("should be able to sign and verify a blind word", async () => {
    const { isValid, isValidOther } = await testingPage.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const otherSecretKey = window.SecretKey.withRng();
      const message = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
      const signingInputs = window.SigningInputs.newBlind(message);
      const signature = signingInputs.sign(secretKey);
      const isValid = signingInputs.verify(secretKey.publicKey(), signature);
      const isValidOther = signingInputs.verify(
        otherSecretKey.publicKey(),
        signature
      );

      return { isValid, isValidOther };
    });
    expect(isValid).to.be.true;
    expect(isValidOther).to.be.false;
  });
});
