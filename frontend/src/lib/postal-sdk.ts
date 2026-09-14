export type SearchMode = "exact" | "prefix" | "partial";

export type PostalCodeRecord = {
  zip_code: string;
  prefecture_id: number;
  city_id: string;
  prefecture: string;
  city: string;
  town: string;
};

export type PrefectureRecord = {
  prefecture_id: number;
  prefecture: string;
};

export type CityRecord = {
  city_id: string;
  city: string;
};

export type PostalRequestOptions = {
  signal?: AbortSignal;
};

export type PostalSearchOptions = PostalRequestOptions & {
  mode?: SearchMode;
  limit?: number;
};

export type PostalSdkOptions = {
  baseUrl?: string;
  fetcher?: typeof fetch;
  timeoutMs?: number;
};

const DEFAULT_API_BASE =
  process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3202";

function clampLimit(limit?: number): number {
  if (!limit || Number.isNaN(limit)) {
    return 50;
  }
  return Math.max(1, Math.min(200, Math.floor(limit)));
}

export function normalizeZip(zipInput: string): string {
  // Only normalize presentation; preserve extra digits and unexpected characters.
  return zipInput
    .replace(/[０-９]/g, (digit) =>
      String.fromCharCode(digit.charCodeAt(0) - 0xfee0),
    )
    .replace(/[\s\-－−]/g, "");
}

export function isValidZip(zipInput: string): boolean {
  return /^[0-9]{7}$/.test(normalizeZip(zipInput));
}

export class PostalSdk {
  private readonly baseUrl: string;
  private readonly fetcher: typeof fetch;
  private readonly timeoutMs: number;

  constructor(options: PostalSdkOptions = {}) {
    this.timeoutMs = options.timeoutMs ?? 10_000;
    if (
      !Number.isInteger(this.timeoutMs) ||
      this.timeoutMs < 1 ||
      this.timeoutMs > 2_147_483_647
    ) {
      throw new RangeError(
        "timeoutMs must be an integer between 1 and 2147483647",
      );
    }
    this.baseUrl = (options.baseUrl ?? DEFAULT_API_BASE).replace(/\/$/, "");
    const fetcher = options.fetcher ?? globalThis.fetch;
    this.fetcher =
      fetcher === globalThis.fetch
        ? globalThis.fetch.bind(globalThis)
        : fetcher;
  }

  async lookupZip(
    zipInput: string,
    options: PostalRequestOptions = {},
  ): Promise<PostalCodeRecord[]> {
    const zip = normalizeZip(zipInput);
    if (!isValidZip(zip)) {
      return [];
    }
    return this.request<PostalCodeRecord[]>(`/postal_codes/${zip}`, options);
  }

  async searchAddress(
    address: string,
    options: PostalSearchOptions = {},
  ): Promise<PostalCodeRecord[]> {
    const keyword = address.trim();
    if (!keyword) {
      return [];
    }

    const mode = options.mode ?? "partial";
    const limit = clampLimit(options.limit);
    const query = new URLSearchParams({
      address: keyword,
      mode,
      limit: String(limit),
    });
    return this.request<PostalCodeRecord[]>(
      `/postal_codes/search?${query.toString()}`,
      options,
    );
  }

  async listPrefectures(
    options: PostalRequestOptions = {},
  ): Promise<PrefectureRecord[]> {
    return this.request<PrefectureRecord[]>(
      "/postal_codes/prefectures",
      options,
    );
  }

  async listCities(
    prefectureId: number,
    options: PostalRequestOptions = {},
  ): Promise<CityRecord[]> {
    const query = new URLSearchParams({ prefecture_id: String(prefectureId) });
    return this.request<CityRecord[]>(
      `/postal_codes/cities?${query.toString()}`,
      options,
    );
  }

  private async request<T>(
    path: string,
    { signal }: PostalRequestOptions,
  ): Promise<T> {
    const controller = new AbortController();
    const cancel = () => controller.abort(signal?.reason);
    if (signal?.aborted) {
      throw signal.reason;
    }
    signal?.addEventListener("abort", cancel, { once: true });
    const timer = setTimeout(() => {
      controller.abort(
        new DOMException("Postal API request timed out", "TimeoutError"),
      );
    }, this.timeoutMs);
    try {
      const response = await this.fetcher(`${this.baseUrl}${path}`, {
        signal: controller.signal,
      });
      if (!response.ok) {
        const errorBody = await response.text();
        throw new Error(
          `Postal API request failed (${response.status}): ${errorBody || response.statusText}`,
        );
      }
      return (await response.json()) as T;
    } catch (error) {
      // Body reads can reject with AbortError even when the cause was a timeout.
      throw controller.signal.aborted ? controller.signal.reason : error;
    } finally {
      clearTimeout(timer);
      signal?.removeEventListener("abort", cancel);
    }
  }
}

export function createPostalSdk(options?: PostalSdkOptions): PostalSdk {
  return new PostalSdk(options);
}

export function formatZip(zipInput: string): string {
  const zip = normalizeZip(zipInput);
  if (zip.length <= 3 || !/^[0-9]+$/.test(zip)) {
    return zip;
  }
  return `${zip.slice(0, 3)}-${zip.slice(3)}`;
}
