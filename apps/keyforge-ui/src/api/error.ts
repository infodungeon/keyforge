export class BackendError extends Error {
  code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = "BackendError";
    this.code = code;
  }

  static from(e: any): Error {
    if (e instanceof Error) return e;
    if (typeof e === "string") {
        // Try parsing JSON string
        try {
            const parsed = JSON.parse(e);
            if (parsed.code && parsed.message) {
                return new BackendError(parsed.code, parsed.message);
            }
        } catch {}
        return new Error(e);
    }
    if (typeof e === "object" && e !== null) {
      if (e.code && e.message) {
        return new BackendError(e.code, e.message);
      }
      return new Error(JSON.stringify(e));
    }
    return new Error("Unknown Error");
  }
}
