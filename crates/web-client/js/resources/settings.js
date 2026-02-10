export class SettingsResource {
  #inner;
  #client;

  constructor(inner, _getWasm, client) {
    this.#inner = inner;
    this.#client = client;
  }

  async get(key) {
    this.#client.assertNotTerminated();
    const value = await this.#inner.getSetting(key);
    return value === undefined ? null : value;
  }

  async set(key, value) {
    this.#client.assertNotTerminated();
    await this.#inner.setSetting(key, value);
  }

  async remove(key) {
    this.#client.assertNotTerminated();
    await this.#inner.removeSetting(key);
  }

  async listKeys() {
    this.#client.assertNotTerminated();
    return await this.#inner.listSettingKeys();
  }
}
