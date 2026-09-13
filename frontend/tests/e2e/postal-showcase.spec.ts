import { expect, Page, test } from "@playwright/test";

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
    await route.fulfill({ json: address === "西新宿" ? fixtures.nishishinjuku : [] });
  });
}

async function expectNoHorizontalOverflow(page: Page) {
  const width = await page.evaluate(() => ({
    body: document.body.scrollWidth,
    html: document.documentElement.scrollWidth,
    viewport: document.documentElement.clientWidth,
  }));

  expect(Math.max(width.body, width.html)).toBeLessThanOrEqual(width.viewport + 1);
}

test.beforeEach(async ({ page }) => {
  await mockPostalApi(page);
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "郵便番号入力を、今どきのUXへ。" })).toBeVisible();
});

test("EC form completes address fields from a postal code", async ({ page }) => {
  await page.getByLabel("郵便番号").fill("100-0001");
  await page.getByRole("button", { name: "住所を補完" }).click();

  await expect(page.getByText("1件ヒットしました。候補から選択できます。")).toBeVisible();
  await expect(page.getByLabel("都道府県")).toHaveValue("東京都");
  await expect(page.getByLabel("市区町村")).toHaveValue("千代田区");
  await expect(page.getByLabel("町域")).toHaveValue("千代田");
});

test("member form searches by town keyword and applies a selected candidate", async ({ page }) => {
  await page.getByRole("button", { name: /会員登録フォーム/ }).click();
  await expect(page.getByText("会員登録フォーム補完サンプル")).toBeVisible();
  await page.getByLabel("住所キーワード検索").fill("西新宿");
  await page.getByRole("button", { name: "キーワード検索", exact: true }).click();

  await expect(page.getByText("2件ヒットしました。候補を選択してください。")).toBeVisible();
  await page.getByRole("button", { name: "東京都新宿区西新宿 (160-0023)", exact: true }).click();
  await expect(page.getByLabel("郵便番号")).toHaveValue("160-0023");
  await expect(page.getByLabel("都道府県")).toHaveValue("東京都");
  await expect(page.getByLabel("市区町村")).toHaveValue("新宿区");
  await expect(page.getByLabel("町域")).toHaveValue("西新宿");
});

test("call center form completes Osaka Umeda from a postal code", async ({ page }) => {
  await page.getByRole("button", { name: /コールセンター入力/ }).click();
  await expect(page.getByText("コールセンター入力支援サンプル")).toBeVisible();
  await page.getByLabel("郵便番号").fill("5300001");
  await page.getByRole("button", { name: "郵便番号検索", exact: true }).click();

  await expect(page.getByText("1件ヒット。候補を選択してください。")).toBeVisible();
  await expect(page.getByLabel("都道府県")).toHaveValue("大阪府");
  await expect(page.getByLabel("市区町村")).toHaveValue("大阪市北区");
  await expect(page.getByLabel("町域")).toHaveValue("梅田");
});

test("short postal code shows validation without calling the API", async ({ page }) => {
  await page.getByLabel("郵便番号").fill("123");
  await page.getByRole("button", { name: "住所を補完" }).click();

  await expect(page.getByText("郵便番号は7桁で入力してください")).toBeVisible();
});

test("layout has no horizontal overflow", async ({ page }) => {
  await expectNoHorizontalOverflow(page);
});
