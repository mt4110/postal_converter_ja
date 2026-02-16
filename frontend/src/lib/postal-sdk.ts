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

export type PostalSearchOptions = {
  mode?: SearchMode;
  limit?: number;
};

export type PostalSdkOptions = {
  baseUrl?: string;
  fetcher?: typeof fetch;
};

const DEFAULT_API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3202";

function clampLimit(limit?: number): number {
  if (!limit || Number.isNaN(limit)) {
    return 50;
  }
  return Math.max(1, Math.min(200, Math.floor(limit)));
}

function normalizeZip(zipInput: string): string {
  return zipInput.replace(/\D/g, "").slice(0, 7);
}

export class PostalSdk {
  private readonly baseUrl: string;
  private readonly fetcher: typeof fetch;

  constructor(options: PostalSdkOptions = {}) {
    this.baseUrl = (options.baseUrl ?? DEFAULT_API_BASE).replace(/\/$/, "");
    const fetcher = options.fetcher ?? globalThis.fetch;
    this.fetcher = fetcher === globalThis.fetch ? globalThis.fetch.bind(globalThis) : fetcher;
  }

  async lookupZip(zipInput: string): Promise<PostalCodeRecord[]> {
    const zip = normalizeZip(zipInput);
    if (zip.length !== 7) {
      return [];
    }
    return this.request<PostalCodeRecord[]>(`/postal_codes/${zip}`);
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
    return this.request<PostalCodeRecord[]>(`/postal_codes/search?${query.toString()}`);
  }

  async listPrefectures(): Promise<PrefectureRecord[]> {
    return this.request<PrefectureRecord[]>("/postal_codes/prefectures");
  }

  async listCities(prefectureId: number): Promise<CityRecord[]> {
    const query = new URLSearchParams({ prefecture_id: String(prefectureId) });
    return this.request<CityRecord[]>(`/postal_codes/cities?${query.toString()}`);
  }

  private async request<T>(path: string): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`);
    if (!response.ok) {
      const errorBody = await response.text();
      throw new Error(
        `Postal API request failed (${response.status}): ${errorBody || response.statusText}`,
      );
    }
    return (await response.json()) as T;
  }
}

export function createPostalSdk(options?: PostalSdkOptions): PostalSdk {
  return new PostalSdk(options);
}

export function formatZip(zipInput: string): string {
  const zip = normalizeZip(zipInput);
  if (zip.length <= 3) {
    return zip;
  }
  return `${zip.slice(0, 3)}-${zip.slice(3)}`;
}
