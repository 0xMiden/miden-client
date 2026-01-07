import test from "./playwright.global.setup";
import { expect } from "@playwright/test";

test.describe("signature", () => {
  [
    ["rpoFalconWithRNG", "Falcon Scheme"],
    ["ecdsaWithRNG", "ECDSA Scheme"],
  ].forEach(([signatureFunction, signatureScheme]) => {
    test(`should produce a valid signature: ${signatureScheme}`, async ({
      page,
    }) => {
      const isValid = await page.evaluate(
        ({ _signatureScheme }) => {
          const sig = "ecdsaWithRNG";
          const secretKey = window.AuthSecretKey[_signatureScheme]();
          const message = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
          const signature = secretKey.sign(message);
          const isValid = secretKey.publicKey().verify(message, signature);

          return isValid;
        },
        { _signatureScheme: signatureFunction }
      );

      expect(isValid).toEqual(true);
    });

    test(`should not verify the wrong message: ${signatureScheme}`, async ({
      page,
    }) => {
      const isValid = await page.evaluate(
        ({ _signatureScheme }) => {
          const secretKey = window.AuthSecretKey[_signatureScheme]();
          const message = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
          const wrongMessage = new window.Word(
            new BigUint64Array([5n, 6n, 7n, 8n])
          );
          const signature = secretKey.sign(message);
          const isValid = secretKey.publicKey().verify(wrongMessage, signature);

          return isValid;
        },
        { _signatureScheme: signatureFunction }
      );
      expect(isValid).toEqual(false);
    });

    test(`should not verify the signature of a different key: ${signatureScheme}`, async ({
      page,
    }) => {
      const isValid = await page.evaluate(
        ({ _signatureScheme }) => {
          const secretKey = window.AuthSecretKey[_signatureScheme]();
          const message = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
          const signature = secretKey.sign(message);
          const differentSecretKey = window.AuthSecretKey[_signatureScheme]();
          const isValid = differentSecretKey
            .publicKey()
            .verify(message, signature);

          return isValid;
        },
        { _signatureScheme: signatureFunction }
      );
      expect(isValid).toEqual(false);
    });

    test(`should be able to serialize and deserialize a signature: ${signatureScheme}`, async ({
      page,
    }) => {
      const isValid = await page.evaluate(
        ({ _signatureScheme }) => {
          const secretKey = window.AuthSecretKey[_signatureScheme]();
          const message = new window.Word(new BigUint64Array([1n, 2n, 3n, 4n]));
          const signature = secretKey.sign(message);
          const serializedSignature = signature.serialize();
          const deserializedSignature =
            window.Signature.deserialize(serializedSignature);

          const isValid = secretKey
            .publicKey()
            .verify(message, deserializedSignature);

          return isValid;
        },
        { _signatureScheme: signatureFunction }
      );
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
      page,
    }) => {
      const isValid = await page.evaluate(
        ({ _signatureScheme }) => {
          const secretKey = window.AuthSecretKey[_signatureScheme]();
          const publicKey = secretKey.publicKey();
          const serializedPublicKey = publicKey.serialize();
          const deserializedPublicKey =
            window.PublicKey.deserialize(serializedPublicKey);
          const serializedDeserializedPublicKey =
            deserializedPublicKey.serialize();
          return (
            serializedPublicKey.toString() ===
            serializedDeserializedPublicKey.toString()
          );
        },
        { _signatureScheme: signatureFunction }
      );
      expect(isValid).toEqual(true);
    });
  });
});

test.describe("signing inputs", () => {
  [
    ["rpoFalconWithRNG", "Falcon Scheme"],
    ["ecdsaWithRNG", "ECDSA Scheme"],
  ].forEach(([signatureFunction, signatureScheme]) => {
    test(`should be able to sign and verify an arbitrary array of felts: ${signatureScheme}`, async ({
      page,
    }) => {
      const { isValid, isValidOther } = await page.evaluate(
        ({ _signatureScheme }) => {
          const secretKey = window.AuthSecretKey[_signatureScheme]();
          const otherSecretKey = window.AuthSecretKey[_signatureScheme]();
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
        },
        { _signatureScheme: signatureFunction }
      );
      expect(isValid).toBe(true);
      expect(isValidOther).toBe(false);
    });

    test(`should be able to sign and verify a blind word: ${signatureScheme}`, async ({
      page,
    }) => {
      const { isValid, isValidOther } = await page.evaluate(
        ({ _signatureScheme }) => {
          const secretKey = window.AuthSecretKey[_signatureScheme]();
          const otherSecretKey = window.AuthSecretKey[_signatureScheme]();
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
        },
        { _signatureScheme: signatureFunction }
      );
      expect(isValid).toBe(true);
      expect(isValidOther).toBe(false);
    });
  });
});
