export class ArchivePreviewRequests<T> {
  private pending = new Map<string, Promise<T>>();

  async run(key: string, valid: () => boolean, start: () => Promise<T>): Promise<T | null> {
    await new Promise(resolve => setTimeout(resolve, 40));
    if (!valid()) return null;
    let request = this.pending.get(key);
    if (!request) {
      request = start();
      this.pending.set(key, request);
      void request.finally(() => {
        if (this.pending.get(key) === request) this.pending.delete(key);
      }).catch(() => {});
    }
    const result = await request;
    return valid() ? result : null;
  }
}
