export class TagsResource {
  #inner;
  #client;

  constructor(inner, getWasm, client) {
    this.#inner = inner;
    this.#client = client;
  }

  async add(tag) {
    this.#client.assertNotTerminated();
    await this.#inner.addTag(String(tag));
  }

  async remove(tag) {
    this.#client.assertNotTerminated();
    await this.#inner.removeTag(String(tag));
  }

  async list() {
    this.#client.assertNotTerminated();
    const tags = await this.#inner.listTags();
    return Array.from(tags).map(Number);
  }
}
