// Mock Redis for development when ioredis is not available
export class MockRedis {
  private store: Map<string, string> = new Map();
  private expirations: Map<string, number> = new Map();

  constructor(_config?: any) {
    console.log('Using mock Redis for development');
  }

  async get(key: string): Promise<string | null> {
    if (this.expirations.has(key) && this.expirations.get(key)! < Date.now()) {
      this.store.delete(key);
      this.expirations.delete(key);
      return null;
    }
    return this.store.get(key) || null;
  }

  async setex(key: string, ttl: number, value: string): Promise<void> {
    this.store.set(key, value);
    this.expirations.set(key, Date.now() + ttl * 1000);
  }

  async quit(): Promise<void> {
    this.store.clear();
    this.expirations.clear();
  }
}