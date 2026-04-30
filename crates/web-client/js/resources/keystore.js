export class KeystoreResource {
  #inner;
  #client;

  constructor(inner, client) {
    this.#inner = inner;
    this.#client = client;
  }

  async insert(accountId, secretKey) {
    this.#client.assertNotTerminated();
    if (this.#inner.keystore) {
      return await this.#inner.keystore.insert(accountId, secretKey);
    }
    return await this.#inner.addAccountSecretKeyToWebStore(
      accountId,
      secretKey
    );
  }

  async get(pubKeyCommitment) {
    this.#client.assertNotTerminated();
    if (this.#inner.keystore) {
      return await this.#inner.keystore.get(pubKeyCommitment);
    }
    return await this.#inner.getAccountAuthByPubKeyCommitment(pubKeyCommitment);
  }

  async remove(pubKeyCommitment) {
    this.#client.assertNotTerminated();
    if (this.#inner.keystore) {
      return await this.#inner.keystore.remove(pubKeyCommitment);
    }
    throw new Error("remove() is not supported on this platform");
  }

  async getCommitments(accountId) {
    this.#client.assertNotTerminated();
    if (this.#inner.keystore) {
      return await this.#inner.keystore.getCommitments(accountId);
    }
    return await this.#inner.getPublicKeyCommitmentsOfAccount(accountId);
  }

  async getAccountId(pubKeyCommitment) {
    this.#client.assertNotTerminated();
    if (this.#inner.keystore) {
      return await this.#inner.keystore.getAccountId(pubKeyCommitment);
    }
    const account =
      await this.#inner.getAccountByKeyCommitment(pubKeyCommitment);
    return account ? account.id() : undefined;
  }
}
