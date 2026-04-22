"use client";

import { AnimatePresence, motion } from "framer-motion";
import {
  Building2,
  Headset,
  IdCard,
  MapPinned,
  Phone,
  Search,
  ShoppingCart,
  Sparkles,
  Truck,
} from "lucide-react";
import { ReactNode, useMemo, useState } from "react";
import { createPostalSdk, formatZip, PostalCodeRecord } from "@/lib/postal-sdk";

type ActiveTab = "ec" | "member" | "callcenter";

type EcFormState = {
  customerName: string;
  phone: string;
  zipCode: string;
  prefecture: string;
  city: string;
  town: string;
  street: string;
  building: string;
};

type MemberFormState = {
  fullName: string;
  email: string;
  zipCode: string;
  prefecture: string;
  city: string;
  town: string;
  addressDetail: string;
};

type CallCenterFormState = {
  customerId: string;
  phone: string;
  zipCode: string;
  keyword: string;
  prefecture: string;
  city: string;
  town: string;
  note: string;
};

const tabItems: Array<{
  id: ActiveTab;
  label: string;
  icon: ReactNode;
  description: string;
}> = [
  {
    id: "ec",
    label: "EC 配送フォーム",
    icon: <ShoppingCart className="h-4 w-4" />,
    description: "郵便番号から配送先を瞬時に補完",
  },
  {
    id: "member",
    label: "会員登録フォーム",
    icon: <IdCard className="h-4 w-4" />,
    description: "住所キーワード検索も併用できる登録導線",
  },
  {
    id: "callcenter",
    label: "コールセンター入力",
    icon: <Headset className="h-4 w-4" />,
    description: "通話中に住所候補を即提示し、入力時間を短縮",
  },
];

const panelAnimation = {
  initial: { opacity: 0, y: 18 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -12 },
  transition: { duration: 0.22, ease: "easeOut" as const },
};

const demoPostalRecords: PostalCodeRecord[] = [
  {
    zip_code: "1000001",
    prefecture_id: 13,
    city_id: "13101",
    prefecture: "東京都",
    city: "千代田区",
    town: "千代田",
  },
  {
    zip_code: "1500002",
    prefecture_id: 13,
    city_id: "13113",
    prefecture: "東京都",
    city: "渋谷区",
    town: "渋谷",
  },
  {
    zip_code: "1600023",
    prefecture_id: 13,
    city_id: "13104",
    prefecture: "東京都",
    city: "新宿区",
    town: "西新宿",
  },
  {
    zip_code: "5300001",
    prefecture_id: 27,
    city_id: "27127",
    prefecture: "大阪府",
    city: "大阪市北区",
    town: "梅田",
  },
  {
    zip_code: "0600001",
    prefecture_id: 1,
    city_id: "01101",
    prefecture: "北海道",
    city: "札幌市中央区",
    town: "北一条西",
  },
];

function normalizeZipInput(value: string): string {
  return value.replace(/\D/g, "").slice(0, 7);
}

function lookupDemoZip(zipInput: string): PostalCodeRecord[] {
  const zip = normalizeZipInput(zipInput);
  return demoPostalRecords.filter((record) => record.zip_code === zip);
}

function searchDemoAddress(keyword: string, limit: number): PostalCodeRecord[] {
  const normalizedKeyword = keyword.replace(/\s+/g, "");
  return demoPostalRecords
    .filter((record) =>
      `${record.prefecture}${record.city}${record.town}`.includes(
        normalizedKeyword,
      ),
    )
    .slice(0, limit);
}

function applyAddressFromRecord<
  T extends { zipCode: string; prefecture: string; city: string; town: string },
>(current: T, record: PostalCodeRecord): T {
  return {
    ...current,
    zipCode: record.zip_code,
    prefecture: record.prefecture,
    city: record.city,
    town: record.town,
  };
}

export default function PostalShowcase() {
  const sdk = useMemo(() => createPostalSdk(), []);
  const demoMode = process.env.NEXT_PUBLIC_DEMO_MODE === "true";
  const [activeTab, setActiveTab] = useState<ActiveTab>("ec");

  const [ecForm, setEcForm] = useState<EcFormState>({
    customerName: "",
    phone: "",
    zipCode: "",
    prefecture: "",
    city: "",
    town: "",
    street: "",
    building: "",
  });
  const [ecCandidates, setEcCandidates] = useState<PostalCodeRecord[]>([]);
  const [ecLoading, setEcLoading] = useState(false);
  const [ecMessage, setEcMessage] = useState<string>("");

  const [memberForm, setMemberForm] = useState<MemberFormState>({
    fullName: "",
    email: "",
    zipCode: "",
    prefecture: "",
    city: "",
    town: "",
    addressDetail: "",
  });
  const [memberKeyword, setMemberKeyword] = useState("");
  const [memberCandidates, setMemberCandidates] = useState<PostalCodeRecord[]>(
    [],
  );
  const [memberLoading, setMemberLoading] = useState(false);
  const [memberMessage, setMemberMessage] = useState<string>("");

  const [callCenterForm, setCallCenterForm] = useState<CallCenterFormState>({
    customerId: "",
    phone: "",
    zipCode: "",
    keyword: "",
    prefecture: "",
    city: "",
    town: "",
    note: "",
  });
  const [callCenterCandidates, setCallCenterCandidates] = useState<
    PostalCodeRecord[]
  >([]);
  const [callCenterLoading, setCallCenterLoading] = useState(false);
  const [callCenterMessage, setCallCenterMessage] = useState("");

  const handleEcZipLookup = async () => {
    if (ecForm.zipCode.length !== 7) {
      setEcMessage("郵便番号は7桁で入力してください");
      return;
    }

    setEcLoading(true);
    setEcMessage("");
    try {
      const rows = demoMode
        ? lookupDemoZip(ecForm.zipCode)
        : await sdk.lookupZip(ecForm.zipCode);
      if (rows.length === 0) {
        setEcCandidates([]);
        setEcMessage("該当する住所が見つかりませんでした");
        return;
      }

      setEcCandidates(rows);
      setEcForm((prev) => applyAddressFromRecord(prev, rows[0]));
      setEcMessage(
        demoMode
          ? `${rows.length}件ヒットしました。GitHub Pages用のデモデータです。`
          : `${rows.length}件ヒットしました。候補から選択できます。`,
      );
    } catch (error) {
      console.error(error);
      setEcMessage("住所補完に失敗しました。API接続を確認してください。");
    } finally {
      setEcLoading(false);
    }
  };

  const handleMemberZipLookup = async () => {
    if (memberForm.zipCode.length !== 7) {
      setMemberMessage("郵便番号は7桁で入力してください");
      return;
    }

    setMemberLoading(true);
    setMemberMessage("");
    try {
      const rows = demoMode
        ? lookupDemoZip(memberForm.zipCode)
        : await sdk.lookupZip(memberForm.zipCode);
      if (rows.length === 0) {
        setMemberCandidates([]);
        setMemberMessage("郵便番号に一致する住所がありません");
        return;
      }

      setMemberCandidates(rows);
      setMemberForm((prev) => applyAddressFromRecord(prev, rows[0]));
      setMemberMessage(
        demoMode
          ? `デモデータから${rows.length}件取得しました`
          : `郵便番号検索で${rows.length}件取得しました`,
      );
    } catch (error) {
      console.error(error);
      setMemberMessage("住所補完に失敗しました。API接続を確認してください。");
    } finally {
      setMemberLoading(false);
    }
  };

  const handleMemberKeywordSearch = async () => {
    const keyword = memberKeyword.trim();
    if (!keyword) {
      setMemberMessage("町名・市区町村名などを入力してください");
      return;
    }

    setMemberLoading(true);
    setMemberMessage("");
    try {
      const rows = demoMode
        ? searchDemoAddress(keyword, 8)
        : await sdk.searchAddress(keyword, { mode: "partial", limit: 8 });
      setMemberCandidates(rows);
      if (rows.length === 0) {
        setMemberMessage("該当候補はありませんでした");
      } else {
        setMemberMessage(
          `${rows.length}件ヒットしました。候補を選択してください。`,
        );
      }
    } catch (error) {
      console.error(error);
      setMemberMessage("住所検索に失敗しました。API接続を確認してください。");
    } finally {
      setMemberLoading(false);
    }
  };

  const handleCallCenterZipLookup = async () => {
    if (callCenterForm.zipCode.length !== 7) {
      setCallCenterMessage("郵便番号は7桁で入力してください");
      return;
    }

    setCallCenterLoading(true);
    setCallCenterMessage("");
    try {
      const rows = demoMode
        ? lookupDemoZip(callCenterForm.zipCode)
        : await sdk.lookupZip(callCenterForm.zipCode);
      setCallCenterCandidates(rows);
      if (rows.length === 0) {
        setCallCenterMessage("候補が見つかりませんでした");
      } else {
        setCallCenterForm((prev) => applyAddressFromRecord(prev, rows[0]));
        setCallCenterMessage(
          demoMode
            ? `${rows.length}件ヒット。デモデータから候補を選択できます。`
            : `${rows.length}件ヒット。候補を選択してください。`,
        );
      }
    } catch (error) {
      console.error(error);
      setCallCenterMessage("検索に失敗しました。API接続を確認してください。");
    } finally {
      setCallCenterLoading(false);
    }
  };

  const handleCallCenterKeywordSearch = async () => {
    const keyword = callCenterForm.keyword.trim();
    if (!keyword) {
      setCallCenterMessage("市区町村・町域などのキーワードを入力してください");
      return;
    }

    setCallCenterLoading(true);
    setCallCenterMessage("");
    try {
      const rows = demoMode
        ? searchDemoAddress(keyword, 10)
        : await sdk.searchAddress(keyword, { mode: "partial", limit: 10 });
      setCallCenterCandidates(rows);
      if (rows.length === 0) {
        setCallCenterMessage("候補がありませんでした");
      } else {
        setCallCenterMessage(
          `${rows.length}件ヒット。通話中に候補を選択してください。`,
        );
      }
    } catch (error) {
      console.error(error);
      setCallCenterMessage("検索に失敗しました。API接続を確認してください。");
    } finally {
      setCallCenterLoading(false);
    }
  };

  return (
    <main className="mx-auto w-full max-w-6xl px-4 pb-20 pt-10 md:px-8 md:pt-14">
      <motion.section
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.35, ease: "easeOut" }}
        className="surface-card relative overflow-hidden p-6 md:p-8"
      >
        <div className="relative z-10">
          <p className="status-chip inline-flex items-center gap-2">
            <motion.span
              animate={{ rotate: [0, 8, -4, 0] }}
              transition={{
                duration: 2.8,
                repeat: Infinity,
                ease: "easeInOut",
              }}
            >
              <Sparkles className="h-4 w-4" />
            </motion.span>
            導入SDKサンプル
          </p>
          <h1 className="mt-4 text-3xl font-bold tracking-tight md:text-4xl">
            郵便番号入力を、今どきのUXへ。
          </h1>
          <p className="mt-3 max-w-3xl text-sm leading-7 text-[color:var(--ink-muted)] md:text-base">
            ECフォーム、会員登録、コールセンター入力支援の3パターンを同じSDKで実装。郵便番号厳密検索と住所キーワード検索を組み合わせ、
            入力工数と誤記を同時に削減できます。
          </p>
          <div className="mt-4 flex flex-wrap items-center gap-2 text-xs text-[color:var(--ink-muted)] md:text-sm">
            <span className="rounded-md bg-black/5 px-2 py-1">
              {demoMode ? "GitHub Pages demo data" : "Live API"}
            </span>
            <span>
              API Base URL:{" "}
              <code className="rounded-md bg-black/5 px-2 py-1">
                {process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3202"}
              </code>
            </span>
          </div>
        </div>
      </motion.section>

      <motion.nav
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.08, duration: 0.3 }}
        className="mt-6 grid gap-3 md:grid-cols-3"
      >
        {tabItems.map((tab) => {
          const active = activeTab === tab.id;
          return (
            <motion.button
              key={tab.id}
              type="button"
              onClick={() => setActiveTab(tab.id)}
              whileHover={{ y: -2 }}
              whileTap={{ scale: 0.99 }}
              className={`surface-card text-left transition-all ${active ? "ring-2 ring-[color:var(--accent)]" : ""}`}
            >
              <div className="flex items-center gap-2 text-sm font-semibold">
                <motion.span
                  animate={
                    active
                      ? { rotate: 8, scale: 1.05 }
                      : { rotate: 0, scale: 1 }
                  }
                  transition={{ duration: 0.2, ease: "easeOut" }}
                >
                  {tab.icon}
                </motion.span>
                {tab.label}
              </div>
              <p className="mt-2 text-xs text-[color:var(--ink-muted)] md:text-sm">
                {tab.description}
              </p>
            </motion.button>
          );
        })}
      </motion.nav>

      <AnimatePresence mode="wait">
        {activeTab === "ec" ? (
          <motion.section
            key="ec-panel"
            {...panelAnimation}
            className="mt-6 surface-card p-6 md:p-8"
          >
            <div className="mb-6 flex items-center gap-2 text-lg font-semibold md:text-xl">
              <Truck className="h-5 w-5" />
              EC 配送先自動補完サンプル
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <label className="field-block md:col-span-2">
                <span className="field-label">郵便番号</span>
                <div className="flex flex-wrap items-center gap-2">
                  <input
                    className="outline-input max-w-[180px]"
                    placeholder="1000001"
                    value={formatZip(ecForm.zipCode)}
                    onChange={(event) =>
                      setEcForm((prev) => ({
                        ...prev,
                        zipCode: normalizeZipInput(event.target.value),
                      }))
                    }
                  />
                  <button
                    type="button"
                    className="pill-button"
                    onClick={handleEcZipLookup}
                    disabled={ecLoading}
                  >
                    <motion.span
                      animate={ecLoading ? { rotate: 360 } : { rotate: 0 }}
                      transition={
                        ecLoading
                          ? { repeat: Infinity, duration: 0.9, ease: "linear" }
                          : { duration: 0.16 }
                      }
                    >
                      <Search className="h-4 w-4" />
                    </motion.span>
                    {ecLoading ? "検索中..." : "住所を補完"}
                  </button>
                </div>
              </label>

              <label className="field-block">
                <span className="field-label">氏名</span>
                <input
                  className="outline-input"
                  value={ecForm.customerName}
                  onChange={(event) =>
                    setEcForm((prev) => ({
                      ...prev,
                      customerName: event.target.value,
                    }))
                  }
                  placeholder="山田 太郎"
                />
              </label>

              <label className="field-block">
                <span className="field-label">電話番号</span>
                <input
                  className="outline-input"
                  value={ecForm.phone}
                  onChange={(event) =>
                    setEcForm((prev) => ({
                      ...prev,
                      phone: event.target.value,
                    }))
                  }
                  placeholder="09012345678"
                />
              </label>

              <label className="field-block">
                <span className="field-label">都道府県</span>
                <input
                  className="outline-input"
                  value={ecForm.prefecture}
                  onChange={(event) =>
                    setEcForm((prev) => ({
                      ...prev,
                      prefecture: event.target.value,
                    }))
                  }
                  placeholder="東京都"
                />
              </label>

              <label className="field-block">
                <span className="field-label">市区町村</span>
                <input
                  className="outline-input"
                  value={ecForm.city}
                  onChange={(event) =>
                    setEcForm((prev) => ({ ...prev, city: event.target.value }))
                  }
                  placeholder="千代田区"
                />
              </label>

              <label className="field-block">
                <span className="field-label">町域</span>
                <input
                  className="outline-input"
                  value={ecForm.town}
                  onChange={(event) =>
                    setEcForm((prev) => ({ ...prev, town: event.target.value }))
                  }
                  placeholder="千代田"
                />
              </label>

              <label className="field-block">
                <span className="field-label">番地</span>
                <input
                  className="outline-input"
                  value={ecForm.street}
                  onChange={(event) =>
                    setEcForm((prev) => ({
                      ...prev,
                      street: event.target.value,
                    }))
                  }
                  placeholder="1-1"
                />
              </label>

              <label className="field-block md:col-span-2">
                <span className="field-label">建物名・部屋番号</span>
                <input
                  className="outline-input"
                  value={ecForm.building}
                  onChange={(event) =>
                    setEcForm((prev) => ({
                      ...prev,
                      building: event.target.value,
                    }))
                  }
                  placeholder="サンプルビル 1201"
                />
              </label>
            </div>

            <div className="mt-4 min-h-6 text-sm text-[color:var(--ink-muted)]">
              {ecMessage}
            </div>
            {ecCandidates.length > 1 ? (
              <motion.div
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                className="candidate-list"
              >
                <div className="mb-2 font-medium">候補を選択</div>
                <div className="grid gap-2">
                  {ecCandidates.map((candidate) => (
                    <motion.button
                      key={`${candidate.zip_code}-${candidate.city_id}-${candidate.town}`}
                      type="button"
                      className="ghost-button justify-start"
                      whileHover={{ x: 2 }}
                      whileTap={{ scale: 0.99 }}
                      onClick={() =>
                        setEcForm((prev) =>
                          applyAddressFromRecord(prev, candidate),
                        )
                      }
                    >
                      <MapPinned className="h-4 w-4" />
                      {candidate.prefecture}
                      {candidate.city}
                      {candidate.town} ({formatZip(candidate.zip_code)})
                    </motion.button>
                  ))}
                </div>
              </motion.div>
            ) : null}
          </motion.section>
        ) : activeTab === "member" ? (
          <motion.section
            key="member-panel"
            {...panelAnimation}
            className="mt-6 surface-card p-6 md:p-8"
          >
            <div className="mb-6 flex items-center gap-2 text-lg font-semibold md:text-xl">
              <IdCard className="h-5 w-5" />
              会員登録フォーム補完サンプル
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <label className="field-block">
                <span className="field-label">氏名</span>
                <input
                  className="outline-input"
                  value={memberForm.fullName}
                  onChange={(event) =>
                    setMemberForm((prev) => ({
                      ...prev,
                      fullName: event.target.value,
                    }))
                  }
                  placeholder="郵便 花子"
                />
              </label>

              <label className="field-block">
                <span className="field-label">メールアドレス</span>
                <input
                  className="outline-input"
                  value={memberForm.email}
                  onChange={(event) =>
                    setMemberForm((prev) => ({
                      ...prev,
                      email: event.target.value,
                    }))
                  }
                  placeholder="hanako@example.com"
                />
              </label>

              <label className="field-block md:col-span-2">
                <span className="field-label">郵便番号</span>
                <div className="flex flex-wrap items-center gap-2">
                  <input
                    className="outline-input max-w-[180px]"
                    placeholder="1600023"
                    value={formatZip(memberForm.zipCode)}
                    onChange={(event) =>
                      setMemberForm((prev) => ({
                        ...prev,
                        zipCode: normalizeZipInput(event.target.value),
                      }))
                    }
                  />
                  <button
                    type="button"
                    className="pill-button"
                    onClick={handleMemberZipLookup}
                    disabled={memberLoading}
                  >
                    <motion.span
                      animate={memberLoading ? { rotate: 360 } : { rotate: 0 }}
                      transition={
                        memberLoading
                          ? { repeat: Infinity, duration: 0.9, ease: "linear" }
                          : { duration: 0.16 }
                      }
                    >
                      <Search className="h-4 w-4" />
                    </motion.span>
                    {memberLoading ? "検索中..." : "郵便番号で補完"}
                  </button>
                </div>
              </label>

              <label className="field-block md:col-span-2">
                <span className="field-label">住所キーワード検索</span>
                <div className="flex flex-wrap items-center gap-2">
                  <input
                    className="outline-input grow"
                    value={memberKeyword}
                    onChange={(event) => setMemberKeyword(event.target.value)}
                    placeholder="例: 新宿 西新宿"
                  />
                  <button
                    type="button"
                    className="pill-button"
                    onClick={handleMemberKeywordSearch}
                    disabled={memberLoading}
                  >
                    <motion.span
                      animate={memberLoading ? { rotate: 360 } : { rotate: 0 }}
                      transition={
                        memberLoading
                          ? { repeat: Infinity, duration: 0.9, ease: "linear" }
                          : { duration: 0.16 }
                      }
                    >
                      <Building2 className="h-4 w-4" />
                    </motion.span>
                    {memberLoading ? "検索中..." : "キーワード検索"}
                  </button>
                </div>
              </label>

              <label className="field-block">
                <span className="field-label">都道府県</span>
                <input
                  className="outline-input"
                  value={memberForm.prefecture}
                  onChange={(event) =>
                    setMemberForm((prev) => ({
                      ...prev,
                      prefecture: event.target.value,
                    }))
                  }
                />
              </label>

              <label className="field-block">
                <span className="field-label">市区町村</span>
                <input
                  className="outline-input"
                  value={memberForm.city}
                  onChange={(event) =>
                    setMemberForm((prev) => ({
                      ...prev,
                      city: event.target.value,
                    }))
                  }
                />
              </label>

              <label className="field-block">
                <span className="field-label">町域</span>
                <input
                  className="outline-input"
                  value={memberForm.town}
                  onChange={(event) =>
                    setMemberForm((prev) => ({
                      ...prev,
                      town: event.target.value,
                    }))
                  }
                />
              </label>

              <label className="field-block">
                <span className="field-label">番地・建物名</span>
                <input
                  className="outline-input"
                  value={memberForm.addressDetail}
                  onChange={(event) =>
                    setMemberForm((prev) => ({
                      ...prev,
                      addressDetail: event.target.value,
                    }))
                  }
                  placeholder="2-8-1"
                />
              </label>
            </div>

            <div className="mt-4 min-h-6 text-sm text-[color:var(--ink-muted)]">
              {memberMessage}
            </div>
            {memberCandidates.length > 0 ? (
              <motion.div
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                className="candidate-list"
              >
                <div className="mb-2 font-medium">検索候補</div>
                <div className="grid gap-2">
                  {memberCandidates.map((candidate) => (
                    <motion.button
                      key={`member-${candidate.zip_code}-${candidate.city_id}-${candidate.town}`}
                      type="button"
                      className="ghost-button justify-start"
                      whileHover={{ x: 2 }}
                      whileTap={{ scale: 0.99 }}
                      onClick={() =>
                        setMemberForm((prev) =>
                          applyAddressFromRecord(prev, candidate),
                        )
                      }
                    >
                      <MapPinned className="h-4 w-4" />
                      {candidate.prefecture}
                      {candidate.city}
                      {candidate.town} ({formatZip(candidate.zip_code)})
                    </motion.button>
                  ))}
                </div>
              </motion.div>
            ) : null}
          </motion.section>
        ) : (
          <motion.section
            key="callcenter-panel"
            {...panelAnimation}
            className="mt-6 surface-card p-6 md:p-8"
          >
            <div className="mb-6 flex items-center gap-2 text-lg font-semibold md:text-xl">
              <Headset className="h-5 w-5" />
              コールセンター入力支援サンプル
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <label className="field-block">
                <span className="field-label">顧客ID</span>
                <input
                  className="outline-input"
                  value={callCenterForm.customerId}
                  onChange={(event) =>
                    setCallCenterForm((prev) => ({
                      ...prev,
                      customerId: event.target.value,
                    }))
                  }
                  placeholder="CUST-2026-0001"
                />
              </label>

              <label className="field-block">
                <span className="field-label">発信者電話番号</span>
                <div className="flex items-center gap-2">
                  <Phone className="h-4 w-4 text-[color:var(--ink-muted)]" />
                  <input
                    className="outline-input"
                    value={callCenterForm.phone}
                    onChange={(event) =>
                      setCallCenterForm((prev) => ({
                        ...prev,
                        phone: event.target.value,
                      }))
                    }
                    placeholder="0312345678"
                  />
                </div>
              </label>

              <label className="field-block">
                <span className="field-label">郵便番号</span>
                <div className="flex flex-wrap items-center gap-2">
                  <input
                    className="outline-input max-w-[180px]"
                    value={formatZip(callCenterForm.zipCode)}
                    onChange={(event) =>
                      setCallCenterForm((prev) => ({
                        ...prev,
                        zipCode: normalizeZipInput(event.target.value),
                      }))
                    }
                    placeholder="5300001"
                  />
                  <button
                    type="button"
                    className="pill-button"
                    onClick={handleCallCenterZipLookup}
                    disabled={callCenterLoading}
                  >
                    <motion.span
                      animate={
                        callCenterLoading ? { rotate: 360 } : { rotate: 0 }
                      }
                      transition={
                        callCenterLoading
                          ? { repeat: Infinity, duration: 0.9, ease: "linear" }
                          : { duration: 0.16 }
                      }
                    >
                      <Search className="h-4 w-4" />
                    </motion.span>
                    {callCenterLoading ? "検索中..." : "郵便番号検索"}
                  </button>
                </div>
              </label>

              <label className="field-block">
                <span className="field-label">住所キーワード</span>
                <div className="flex flex-wrap items-center gap-2">
                  <input
                    className="outline-input"
                    value={callCenterForm.keyword}
                    onChange={(event) =>
                      setCallCenterForm((prev) => ({
                        ...prev,
                        keyword: event.target.value,
                      }))
                    }
                    placeholder="梅田 / 渋谷 など"
                  />
                  <button
                    type="button"
                    className="pill-button"
                    onClick={handleCallCenterKeywordSearch}
                    disabled={callCenterLoading}
                  >
                    <motion.span
                      animate={
                        callCenterLoading ? { rotate: 360 } : { rotate: 0 }
                      }
                      transition={
                        callCenterLoading
                          ? { repeat: Infinity, duration: 0.9, ease: "linear" }
                          : { duration: 0.16 }
                      }
                    >
                      <Building2 className="h-4 w-4" />
                    </motion.span>
                    {callCenterLoading ? "検索中..." : "候補検索"}
                  </button>
                </div>
              </label>

              <label className="field-block">
                <span className="field-label">都道府県</span>
                <input
                  className="outline-input"
                  value={callCenterForm.prefecture}
                  onChange={(event) =>
                    setCallCenterForm((prev) => ({
                      ...prev,
                      prefecture: event.target.value,
                    }))
                  }
                />
              </label>

              <label className="field-block">
                <span className="field-label">市区町村</span>
                <input
                  className="outline-input"
                  value={callCenterForm.city}
                  onChange={(event) =>
                    setCallCenterForm((prev) => ({
                      ...prev,
                      city: event.target.value,
                    }))
                  }
                />
              </label>

              <label className="field-block">
                <span className="field-label">町域</span>
                <input
                  className="outline-input"
                  value={callCenterForm.town}
                  onChange={(event) =>
                    setCallCenterForm((prev) => ({
                      ...prev,
                      town: event.target.value,
                    }))
                  }
                />
              </label>

              <label className="field-block">
                <span className="field-label">オペレーターメモ</span>
                <input
                  className="outline-input"
                  value={callCenterForm.note}
                  onChange={(event) =>
                    setCallCenterForm((prev) => ({
                      ...prev,
                      note: event.target.value,
                    }))
                  }
                  placeholder="再配達希望時間など"
                />
              </label>
            </div>

            <div className="mt-4 min-h-6 text-sm text-[color:var(--ink-muted)]">
              {callCenterMessage}
            </div>
            {callCenterCandidates.length > 0 ? (
              <motion.div
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                className="candidate-list"
              >
                <div className="mb-2 font-medium">通話中候補</div>
                <div className="grid gap-2">
                  {callCenterCandidates.map((candidate) => (
                    <motion.button
                      key={`cc-${candidate.zip_code}-${candidate.city_id}-${candidate.town}`}
                      type="button"
                      className="ghost-button justify-start"
                      whileHover={{ x: 2 }}
                      whileTap={{ scale: 0.99 }}
                      onClick={() =>
                        setCallCenterForm((prev) =>
                          applyAddressFromRecord(prev, candidate),
                        )
                      }
                    >
                      <MapPinned className="h-4 w-4" />
                      {candidate.prefecture}
                      {candidate.city}
                      {candidate.town} ({formatZip(candidate.zip_code)})
                    </motion.button>
                  ))}
                </div>
              </motion.div>
            ) : null}
          </motion.section>
        )}
      </AnimatePresence>

      <motion.section
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.14, duration: 0.32 }}
        className="mt-6 surface-card p-6"
      >
        <h2 className="text-lg font-semibold">導入SDKの最小コード</h2>
        <p className="mt-2 text-sm text-[color:var(--ink-muted)]">
          フォーム側はSDKの `lookupZip` / `searchAddress`
          を呼ぶだけで導入できます。
        </p>
        <pre className="mt-4 overflow-x-auto rounded-xl border border-[color:var(--line)] bg-[#101820] p-4 text-xs leading-6 text-[#d7e4ef] md:text-sm">
          {`import { createPostalSdk } from "@/lib/postal-sdk";

const sdk = createPostalSdk({ baseUrl: process.env.NEXT_PUBLIC_API_URL });

const byZip = await sdk.lookupZip("1600023");
const byKeyword = await sdk.searchAddress("西新宿", { mode: "partial", limit: 8 });`}
        </pre>
      </motion.section>
    </main>
  );
}
