const MAX_BUFFER_SIZE = 100 * 1024; // 100KB per pane

export class PaneBufferManager {
  private buffers = new Map<number, string>();

  append(paneId: number, data: string): void {
    const existing = this.buffers.get(paneId) ?? '';
    let updated = existing + data;
    if (updated.length > MAX_BUFFER_SIZE) {
      updated = updated.slice(updated.length - MAX_BUFFER_SIZE);
    }
    this.buffers.set(paneId, updated);
  }

  get(paneId: number): string {
    return this.buffers.get(paneId) ?? '';
  }

  clear(paneId: number): void {
    this.buffers.delete(paneId);
  }

  clearAll(): void {
    this.buffers.clear();
  }
}
