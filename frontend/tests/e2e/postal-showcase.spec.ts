import { expect, Page, Route, test } from "@playwright/test";

const fixtures = {
  tokyo: [
    {
      zip_code: "1000001",
      prefecture_id: 13,
      city_id: "13101",
      prefecture: "東京都",
      city: "千代田区",
      town: "千代田",
    },
  ],
  nishishinjuku: [
    {
      zip_code: "1600023",
      prefecture_id: 13,
      city_id: "13104",
      prefecture: "東京都",
      city: "新宿区",
      town: "西新宿",
    },
    {
      zip_code: "1638001",
      prefecture_id: 13,
      city_id: "13104",
      prefecture: "東京都",
      city: "新宿区",
      town: "西新宿",
    },
  ],
  osaka: [
    {
      zip_code: "5300001",
      prefecture_id: 27,
      city_id: "27127",
      prefecture: "大阪府",
      city: "大阪市北区",
      town: "梅田",
    },
  ],
};

async function mockPostalApi(page: Page) {
  await page.route("**/postal_codes/1000001", async (route) => {
    await route.fulfill({ json: fixtures.tokyo });
  });
  await page.route("**/postal_codes/5300001", async (route) => {
    await route.fulfill({ json: fixtures.osaka });
  });
  await page.route("**/postal_codes/search?**", async (route) => {
    const url = new URL(route.request().url());
    const address = url.searchParams.get("address");
    await route.fulfill({
      json: address === "西新宿" ? fixtures.nishishinjuku : [],
    });
  });
}

async function expectNoHorizontalOverflow(page: Page) {
  const width = await page.evaluate(() => ({
    body: document.body.scrollWidth,
    html: document.documentElement.scrollWidth,
    viewport: document.documentElement.clientWidth,
  }));

  expect(Math.max(width.body, width.html)).toBeLessThanOrEqual(
    width.viewport + 1,
  );
}

const pageErrors = new WeakMap<Page, string[]>();

test.beforeEach(async ({ page }) => {
  const errors: string[] = [];
  pageErrors.set(page, errors);
  page.on("pageerror", (error) => errors.push(error.message));
  await mockPostalApi(page);
  await page.goto("/");
  await expect(page).toHaveTitle("Postal Converter JA | SDK Samples");
  await expect(
    page.getByRole("heading", { name: "郵便番号入力を、今どきのUXへ。" }),
  ).toBeVisible();
});

test.afterEach(async ({ page }) => {
  expect(pageErrors.get(page)).toEqual([]);
});

test("EC form completes address fields from a postal code", async ({
  page,
}) => {
  await page.getByLabel("郵便番号").fill("100-0001");
  await page.getByRole("button", { name: "住所を補完" }).click();

  await expect(
    page.getByText("1件ヒットしました。候補から選択できます。"),
  ).toBeVisible();
  await expect(page.getByLabel("都道府県")).toHaveValue("東京都");
  await expect(page.getByLabel("市区町村")).toHaveValue("千代田区");
  await expect(page.getByLabel("町域")).toHaveValue("千代田");
});

test("member form searches by town keyword and applies a selected candidate", async ({
  page,
}) => {
  await page.getByRole("button", { name: /会員登録フォーム/ }).click();
  await expect(page.getByText("会員登録フォーム補完サンプル")).toBeVisible();
  await page.getByLabel("住所キーワード検索").fill("西新宿");
  await page
    .getByRole("button", { name: "キーワード検索", exact: true })
    .click();

  await expect(
    page.getByText("2件ヒットしました。候補を選択してください。"),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "東京都新宿区西新宿 (160-0023)", exact: true })
    .click();
  await expect(page.getByLabel("郵便番号")).toHaveValue("160-0023");
  await expect(page.getByLabel("都道府県")).toHaveValue("東京都");
  await expect(page.getByLabel("市区町村")).toHaveValue("新宿区");
  await expect(page.getByLabel("町域")).toHaveValue("西新宿");
});

test("call center form completes Osaka Umeda from a postal code", async ({
  page,
}) => {
  await page.getByRole("button", { name: /コールセンター入力/ }).click();
  await expect(page.getByText("コールセンター入力支援サンプル")).toBeVisible();
  await page.getByLabel("郵便番号").fill("5300001");
  await page.getByRole("button", { name: "郵便番号検索", exact: true }).click();

  await expect(
    page.getByText("1件ヒット。候補を選択してください。"),
  ).toBeVisible();
  await expect(page.getByLabel("都道府県")).toHaveValue("大阪府");
  await expect(page.getByLabel("市区町村")).toHaveValue("大阪市北区");
  await expect(page.getByLabel("町域")).toHaveValue("梅田");
});

test("short postal code shows validation without calling the API", async ({
  page,
}) => {
  await page.getByLabel("郵便番号").fill("123");
  await page.getByRole("button", { name: "住所を補完" }).click();

  await expect(page.getByText("郵便番号は7桁で入力してください")).toBeVisible();
});

test("layout has no horizontal overflow", async ({ page }) => {
  await expectNoHorizontalOverflow(page);
});

const forms = [
  {
    tab: "EC 配送フォーム",
    search: "住所を補完",
    panel: "EC 配送先自動補完サンプル",
  },
  {
    tab: "会員登録フォーム",
    search: "郵便番号で補完",
    panel: "会員登録フォーム補完サンプル",
  },
  {
    tab: "コールセンター入力",
    search: "郵便番号検索",
    panel: "コールセンター入力支援サンプル",
  },
];

for (const form of forms) {
  test(`${form.tab}: accepts full-width postal numbers`, async ({ page }) => {
    await page.getByRole("button", { name: new RegExp(form.tab) }).click();
    await expect(page.getByText(form.panel)).toBeVisible();
    await page.getByLabel("郵便番号").fill("１００－０００１");
    await page.getByRole("button", { name: form.search, exact: true }).click();
    await expect(page.getByLabel("郵便番号")).toHaveValue("100-0001");
    await expect(page.getByLabel("都道府県")).toHaveValue("東京都");
  });

  test(`${form.tab}: rejects extra digits without truncation or API access`, async ({
    page,
  }) => {
    let requests = 0;
    page.on("request", (request) => {
      if (request.url().includes("/postal_codes/")) requests += 1;
    });
    await page.getByRole("button", { name: new RegExp(form.tab) }).click();
    await expect(page.getByText(form.panel)).toBeVisible();
    await page.getByLabel("郵便番号").fill("10000012");
    await page.getByRole("button", { name: form.search, exact: true }).click();
    await expect(
      page.getByText("郵便番号は7桁で入力してください"),
    ).toBeVisible();
    await expect(page.getByLabel("郵便番号")).toHaveValue("100-00012");
    await expect(page.getByLabel("都道府県")).toBeEmpty();
    expect(requests).toBe(0);
    await expectNoHorizontalOverflow(page);
  });

  test(`${form.tab}: editing during a lookup cancels the old result and permits retry`, async ({
    page,
  }) => {
    const pending = Promise.withResolvers<Route>();
    await page.route("**/postal_codes/1000001", (route) =>
      pending.resolve(route),
    );
    await page.getByRole("button", { name: new RegExp(form.tab) }).click();
    await expect(page.getByText(form.panel)).toBeVisible();
    await page.getByLabel("郵便番号").fill("1000001");
    await page.getByRole("button", { name: form.search, exact: true }).click();
    const route = await pending.promise;
    await page.getByLabel("郵便番号").fill("5300001");
    await expect(
      page.getByRole("button", { name: form.search, exact: true }),
    ).toBeEnabled();
    await page.getByRole("button", { name: form.search, exact: true }).click();
    await expect(page.getByLabel("都道府県")).toHaveValue("大阪府");
    await route.fulfill({ json: fixtures.tokyo });
    await expect(page.getByLabel("郵便番号")).toHaveValue("530-0001");
    await expect(page.getByLabel("都道府県")).toHaveValue("大阪府");
    await expect(page.getByText(/1件ヒット|郵便番号検索で1件/)).toBeVisible();
  });

  test(`${form.tab}: a manual address edit is not overwritten by a pending lookup`, async ({
    page,
  }) => {
    const pending = Promise.withResolvers<Route>();
    await page.route("**/postal_codes/1000001", (route) =>
      pending.resolve(route),
    );
    await page.getByRole("button", { name: new RegExp(form.tab) }).click();
    await expect(page.getByText(form.panel)).toBeVisible();
    await page.getByLabel("郵便番号").fill("1000001");
    await page.getByRole("button", { name: form.search, exact: true }).click();
    const route = await pending.promise;
    await page.getByLabel("町域").fill("手入力した町域");
    await route.fulfill({ json: fixtures.tokyo });
    await expect(
      page.getByRole("button", { name: form.search, exact: true }),
    ).toBeEnabled();
    await expect(page.getByLabel("町域")).toHaveValue("手入力した町域");
    await expect(page.getByLabel("都道府県")).toBeEmpty();
  });
}

for (const tab of ["会員登録フォーム", "コールセンター入力"]) {
  test(`${tab}: editing the keyword removes pending candidates`, async ({
    page,
  }) => {
    const pending = Promise.withResolvers<Route>();
    await page.route("**/postal_codes/search?**", (route) =>
      pending.resolve(route),
    );
    await page.getByRole("button", { name: new RegExp(tab) }).click();
    const keyword = page.getByLabel(
      tab === "会員登録フォーム" ? "住所キーワード検索" : "住所キーワード",
    );
    await keyword.fill("西新宿");
    await page
      .getByRole("button", {
        name: tab === "会員登録フォーム" ? "キーワード検索" : "候補検索",
        exact: true,
      })
      .click();
    const route = await pending.promise;
    await keyword.fill("梅田");
    await route.fulfill({ json: fixtures.nishishinjuku });
    await expect(
      page.getByRole("button", {
        name: tab === "会員登録フォーム" ? "キーワード検索" : "候補検索",
        exact: true,
      }),
    ).toBeEnabled();
    await expect(
      page.getByRole("button", { name: /東京都新宿区西新宿/ }),
    ).toHaveCount(0);
    await expect(keyword).toHaveValue("梅田");
  });
}

for (const failure of ["http", "network", "json"]) {
  test(`failed ${failure} search clears old candidates and allows retry`, async ({
    page,
  }) => {
    await page.getByRole("button", { name: /会員登録フォーム/ }).click();
    await page.getByLabel("住所キーワード検索").fill("西新宿");
    const search = page.getByRole("button", {
      name: "キーワード検索",
      exact: true,
    });
    const candidate = page.getByRole("button", {
      name: "東京都新宿区西新宿 (160-0023)",
      exact: true,
    });
    await search.click();
    await expect(candidate).toBeVisible();
    await page.route(
      "**/postal_codes/search?**",
      async (route) => {
        if (failure === "network") await route.abort("failed");
        else if (failure === "json")
          await route.fulfill({
            body: "{broken",
            contentType: "application/json",
          });
        else await route.fulfill({ status: 503, body: "Unavailable" });
      },
      { times: 1 },
    );
    await search.click();
    await expect(page.getByRole("status")).toContainText(
      "住所検索に失敗しました",
    );
    await expect(candidate).toHaveCount(0);
    await expect(search).toBeEnabled();
    await expectNoHorizontalOverflow(page);
    await search.click();
    await expect(candidate).toBeVisible();
  });
}

test("timeout restores the search button and permits a successful retry", async ({
  page,
}) => {
  await page.clock.install();
  const pending = Promise.withResolvers<Route>();
  await page.route(
    "**/postal_codes/1000001",
    (route) => pending.resolve(route),
    { times: 1 },
  );
  await page.getByLabel("郵便番号").fill("1000001");
  await page.getByRole("button", { name: "住所を補完" }).click();
  const route = await pending.promise;
  await page.clock.fastForward(10_001);
  await expect(page.getByRole("status")).toContainText(
    "検索がタイムアウトしました",
  );
  await expect(page.getByRole("button", { name: "住所を補完" })).toBeEnabled();
  await expect(page.getByLabel("都道府県")).toBeEmpty();
  await expectNoHorizontalOverflow(page);
  await route.fulfill({ json: fixtures.tokyo });
  await page.getByRole("button", { name: "住所を補完" }).click();
  await expect(page.getByLabel("都道府県")).toHaveValue("東京都");
});
