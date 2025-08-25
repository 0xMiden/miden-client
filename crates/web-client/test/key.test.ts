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

describe("signing inputs", () => {
  it("should be able to sign and verify an arbitrary array of felts", async () => {
    const { isValid, isValidOther } = await testingPage.evaluate(() => {
      const secretKey = window.SecretKey.withRng();
      const otherSecretKey = window.SecretKey.withRng();
      const message = Array.from(
        { length: 128 },
        (_, i) => new window.Felt(BigInt(i))
      );
      const signingInputs = window.SigningInputs.newArbitrary(message);
      const signature = secretKey.signData(signingInputs);
      const isValid = secretKey
        .publicKey()
        .verifyData(signingInputs, signature);
      const isValidOther = otherSecretKey
        .publicKey()
        .verifyData(signingInputs, signature);

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
      const signature = secretKey.signData(signingInputs);
      const isValid = secretKey
        .publicKey()
        .verifyData(signingInputs, signature);
      const isValidOther = otherSecretKey
        .publicKey()
        .verifyData(signingInputs, signature);

      return { isValid, isValidOther };
    });
    expect(isValid).to.be.true;
    expect(isValidOther).to.be.false;
  });
});
