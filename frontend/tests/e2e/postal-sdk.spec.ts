import { expect, test } from "@playwright/test";
import {
  createPostalSdk,
  formatZip,
  isValidZip,
  normalizeZip,
} from "../../src/lib/postal-sdk";

test("SDK normalizes postal presentation without losing invalid input", async () => {
  const urls: string[] = [];
  const sdk = createPostalSdk({
    baseUrl: "http://postal.test",
    fetcher: async (url) => {
      urls.push(String(url));
      return Response.json([]);
    },
  });
  expect(normalizeZip(" １００－０００１　")).toBe("1000001");
  expect(formatZip("１００−０００１")).toBe("100-0001");
  expect(formatZip("10000012")).toBe("100-00012");
  await sdk.lookupZip("１００－０００１");
  for (const input of [
    "",
    "123",
    "10000012",
    "1000001abc",
    "100000!",
    "〒1000001",
  ]) {
    expect(isValidZip(input)).toBe(false);
    await expect(sdk.lookupZip(input)).resolves.toEqual([]);
  }
  expect(urls).toEqual(["http://postal.test/postal_codes/1000001"]);
});

const operations = [
  {
    name: "zip",
    run: (sdk: ReturnType<typeof createPostalSdk>, signal: AbortSignal) =>
      sdk.lookupZip("1000001", { signal }),
  },
  {
    name: "address",
    run: (sdk: ReturnType<typeof createPostalSdk>, signal: AbortSignal) =>
      sdk.searchAddress("西新宿", { signal }),
  },
  {
    name: "prefectures",
    run: (sdk: ReturnType<typeof createPostalSdk>, signal: AbortSignal) =>
      sdk.listPrefectures({ signal }),
  },
  {
    name: "cities",
    run: (sdk: ReturnType<typeof createPostalSdk>, signal: AbortSignal) =>
      sdk.listCities(13, { signal }),
  },
];

for (const operation of operations) {
  test(`SDK ${operation.name} supports cancellation, including pre-aborted signals`, async () => {
    let calls = 0;
    const sdk = createPostalSdk({
      fetcher: async (_url, options) => {
        calls += 1;
        const signal = options!.signal!;
        return new Promise((_resolve, reject) => {
          signal.addEventListener("abort", () => reject(signal.reason), {
            once: true,
          });
        });
      },
    });
    const controller = new AbortController();
    const result = operation.run(sdk, controller.signal);
    const assertion = expect(result).rejects.toHaveProperty(
      "name",
      "AbortError",
    );
    controller.abort();
    await assertion;
    await expect(operation.run(sdk, controller.signal)).rejects.toHaveProperty(
      "name",
      "AbortError",
    );
    expect(calls).toBe(1);
  });
}

test("SDK times out stalled fetches and can retry afterward", async () => {
  let calls = 0;
  const sdk = createPostalSdk({
    timeoutMs: 20,
    fetcher: async (_url, options) => {
      if (++calls > 1) return Response.json([]);
      const signal = options!.signal!;
      return new Promise((_resolve, reject) => {
        signal.addEventListener("abort", () => reject(signal.reason), {
          once: true,
        });
      });
    },
  });
  await expect(sdk.lookupZip("1000001")).rejects.toHaveProperty(
    "name",
    "TimeoutError",
  );
  await expect(sdk.lookupZip("1000001")).resolves.toEqual([]);
});

test("SDK timeout also covers reading a stalled response body", async () => {
  const sdk = createPostalSdk({
    timeoutMs: 20,
    fetcher: async (_url, options) =>
      new Response(
        new ReadableStream({
          start(controller) {
            options!.signal!.addEventListener(
              "abort",
              () => {
                controller.error(
                  new DOMException("Body aborted", "AbortError"),
                );
              },
              { once: true },
            );
          },
        }),
      ),
  });
  await expect(sdk.lookupZip("1000001")).rejects.toHaveProperty(
    "name",
    "TimeoutError",
  );
});

test("SDK clears timeout and detaches cancellation after success", async () => {
  let requestSignal: AbortSignal | null | undefined;
  const caller = new AbortController();
  const sdk = createPostalSdk({
    timeoutMs: 20,
    fetcher: async (_url, options) => {
      requestSignal = options?.signal;
      return Response.json([]);
    },
  });
  await sdk.lookupZip("1000001", { signal: caller.signal });
  caller.abort();
  await new Promise((resolve) => setTimeout(resolve, 40));
  expect(requestSignal?.aborted).toBe(false);
});

test("SDK rejects invalid timeout configuration", () => {
  for (const timeoutMs of [0, -1, 1.5, NaN, Infinity, 2_147_483_648]) {
    expect(() => createPostalSdk({ timeoutMs })).toThrow(RangeError);
  }
});
