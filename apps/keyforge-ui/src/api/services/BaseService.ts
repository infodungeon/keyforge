// apps/keyforge-ui/src/api/services/BaseService.ts

export class BaseService {
  protected cache: Map<string, Promise<any>> = new Map();
  protected hiveUrl: string;

  constructor(hiveUrl: string) {
    this.hiveUrl = hiveUrl;
  }

  setHiveUrl(url: string) {
    this.hiveUrl = url;
    this.cache.clear(); // Clear cache when switching URLs
  }

  protected async fetchJson<T>(
    endpoint: string,
    hiveUrl?: string,
    options?: RequestInit,
  ): Promise<T> {
    const url = hiveUrl || this.hiveUrl;
    const fullUrl = `${url}/${endpoint}`;
    const isGet = !options?.method || options.method === "GET";

    if (isGet) {
      if (this.cache.has(fullUrl)) {
        return this.cache.get(fullUrl) as Promise<T>;
      }
    }

    const promise = (async () => {
      try {
        const res = await fetch(fullUrl, options);
        if (!res.ok) {
          const text = await res.text();
          console.error(`[BACKEND ERROR] ${res.status} ${fullUrl}:`, text);
          throw new Error(`Backend Error (${res.status}): ${text}`);
        }
        return await res.json();
      } catch (e) {
        if (isGet) this.cache.delete(fullUrl);
        console.error(`[FETCH FAILURE] ${fullUrl}:`, e);
        throw e;
      }
    })();

    if (isGet) {
      this.cache.set(fullUrl, promise);
    }

    return promise;
  }
}
