export class KeystoreResource {
  #inner;
  #client;

  constructor(inner, client) {
    this.#inner = inner;
    this.#client = client;
  }

  async insert(accountId, secretKey) {
    this.#client.assertNotTerminated();
    const ks = this.#inner.keystore;
    return await ks.insert(accountId, secretKey);
  }

  async get(pubKeyCommitment) {
    this.#client.assertNotTerminated();
    const ks = this.#inner.keystore;
    return await ks.get(pubKeyCommitment);
  }

  async remove(pubKeyCommitment) {
    this.#client.assertNotTerminated();
    const ks = this.#inner.keystore;
    return await ks.remove(pubKeyCommitment);
  }

  async getCommitments(accountId) {
    this.#client.assertNotTerminated();
    const ks = this.#inner.keystore;
    return await ks.getCommitments(accountId);
  }

  async getAccountId(pubKeyCommitment) {
    this.#client.assertNotTerminated();
    const ks = this.#inner.keystore;
    return await ks.getAccountId(pubKeyCommitment);
  }
}
