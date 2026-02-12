"use client";

import { AnimatePresence, motion } from "framer-motion";
import {
  Building2,
  IdCard,
  MapPinned,
  Search,
  ShoppingCart,
  Sparkles,
  Truck,
} from "lucide-react";
import { ReactNode, useMemo, useState } from "react";
import { createPostalSdk, formatZip, PostalCodeRecord } from "@/lib/postal-sdk";

type ActiveTab = "ec" | "member";

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

const tabItems: Array<{ id: ActiveTab; label: string; icon: ReactNode; description: string }> = [
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
];

const panelAnimation = {
  initial: { opacity: 0, y: 18 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -12 },
  transition: { duration: 0.22, ease: "easeOut" as const },
};

function normalizeZipInput(value: string): string {
  return value.replace(/\D/g, "").slice(0, 7);
}

function applyAddressFromRecord<T extends { zipCode: string; prefecture: string; city: string; town: string }>(
  current: T,
  record: PostalCodeRecord,
): T {
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
  const [memberCandidates, setMemberCandidates] = useState<PostalCodeRecord[]>([]);
  const [memberLoading, setMemberLoading] = useState(false);
  const [memberMessage, setMemberMessage] = useState<string>("");

  const handleEcZipLookup = async () => {
    if (ecForm.zipCode.length !== 7) {
      setEcMessage("郵便番号は7桁で入力してください");
      return;
    }

    setEcLoading(true);
    setEcMessage("");
    try {
      const rows = await sdk.lookupZip(ecForm.zipCode);
      if (rows.length === 0) {
        setEcCandidates([]);
        setEcMessage("該当する住所が見つかりませんでした");
        return;
      }

      setEcCandidates(rows);
      setEcForm((prev) => applyAddressFromRecord(prev, rows[0]));
      setEcMessage(`${rows.length}件ヒットしました。候補から選択できます。`);
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
      const rows = await sdk.lookupZip(memberForm.zipCode);
      if (rows.length === 0) {
        setMemberCandidates([]);
        setMemberMessage("郵便番号に一致する住所がありません");
        return;
      }

      setMemberCandidates(rows);
      setMemberForm((prev) => applyAddressFromRecord(prev, rows[0]));
      setMemberMessage(`郵便番号検索で${rows.length}件取得しました`);
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
      const rows = await sdk.searchAddress(keyword, { mode: "partial", limit: 8 });
      setMemberCandidates(rows);
      if (rows.length === 0) {
        setMemberMessage("該当候補はありませんでした");
      } else {
        setMemberMessage(`${rows.length}件ヒットしました。候補を選択してください。`);
      }
    } catch (error) {
      console.error(error);
      setMemberMessage("住所検索に失敗しました。API接続を確認してください。");
    } finally {
      setMemberLoading(false);
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
        <div className="pointer-events-none absolute -right-12 -top-12 h-40 w-40 rounded-full bg-[rgba(15,118,110,0.18)] blur-2xl" />
        <div className="pointer-events-none absolute -bottom-16 left-8 h-44 w-44 rounded-full bg-[rgba(217,119,6,0.16)] blur-2xl" />
        <div className="relative z-10">
          <p className="status-chip inline-flex items-center gap-2">
            <Sparkles className="h-4 w-4" />
            導入SDKサンプル
          </p>
          <h1 className="mt-4 text-3xl font-bold tracking-tight md:text-4xl">
            郵便番号入力を、今どきのUXへ。
          </h1>
          <p className="mt-3 max-w-3xl text-sm leading-7 text-[color:var(--ink-muted)] md:text-base">
            ECフォームと会員登録フォームの2パターンを同じSDKで実装。郵便番号厳密検索と住所キーワード検索を組み合わせ、
            入力工数と誤記を同時に削減できます。
          </p>
          <div className="mt-4 text-xs text-[color:var(--ink-muted)] md:text-sm">
            API Base URL: <code className="rounded bg-black/5 px-2 py-1">{process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3202"}</code>
          </div>
        </div>
      </motion.section>

      <motion.nav
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.08, duration: 0.3 }}
        className="mt-6 grid gap-3 md:grid-cols-2"
      >
        {tabItems.map((tab) => {
          const active = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              type="button"
              onClick={() => setActiveTab(tab.id)}
              className={`surface-card text-left transition-all ${active ? "ring-2 ring-[color:var(--accent)]" : "hover:-translate-y-0.5"}`}
            >
              <div className="flex items-center gap-2 text-sm font-semibold">
                {tab.icon}
                {tab.label}
              </div>
              <p className="mt-2 text-xs text-[color:var(--ink-muted)] md:text-sm">{tab.description}</p>
            </button>
          );
        })}
      </motion.nav>

      <AnimatePresence mode="wait">
        {activeTab === "ec" ? (
          <motion.section key="ec-panel" {...panelAnimation} className="mt-6 surface-card p-6 md:p-8">
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
                  <button type="button" className="pill-button" onClick={handleEcZipLookup} disabled={ecLoading}>
                    <Search className="h-4 w-4" />
                    {ecLoading ? "検索中..." : "住所を補完"}
                  </button>
                </div>
              </label>

              <label className="field-block">
                <span className="field-label">氏名</span>
                <input
                  className="outline-input"
                  value={ecForm.customerName}
                  onChange={(event) => setEcForm((prev) => ({ ...prev, customerName: event.target.value }))}
                  placeholder="山田 太郎"
                />
              </label>

              <label className="field-block">
                <span className="field-label">電話番号</span>
                <input
                  className="outline-input"
                  value={ecForm.phone}
                  onChange={(event) => setEcForm((prev) => ({ ...prev, phone: event.target.value }))}
                  placeholder="09012345678"
                />
              </label>

              <label className="field-block">
                <span className="field-label">都道府県</span>
                <input
                  className="outline-input"
                  value={ecForm.prefecture}
                  onChange={(event) => setEcForm((prev) => ({ ...prev, prefecture: event.target.value }))}
                  placeholder="東京都"
                />
              </label>

              <label className="field-block">
                <span className="field-label">市区町村</span>
                <input
                  className="outline-input"
                  value={ecForm.city}
                  onChange={(event) => setEcForm((prev) => ({ ...prev, city: event.target.value }))}
                  placeholder="千代田区"
                />
              </label>

              <label className="field-block">
                <span className="field-label">町域</span>
                <input
                  className="outline-input"
                  value={ecForm.town}
                  onChange={(event) => setEcForm((prev) => ({ ...prev, town: event.target.value }))}
                  placeholder="千代田"
                />
              </label>

              <label className="field-block">
                <span className="field-label">番地</span>
                <input
                  className="outline-input"
                  value={ecForm.street}
                  onChange={(event) => setEcForm((prev) => ({ ...prev, street: event.target.value }))}
                  placeholder="1-1"
                />
              </label>

              <label className="field-block md:col-span-2">
                <span className="field-label">建物名・部屋番号</span>
                <input
                  className="outline-input"
                  value={ecForm.building}
                  onChange={(event) => setEcForm((prev) => ({ ...prev, building: event.target.value }))}
                  placeholder="サンプルビル 1201"
                />
              </label>
            </div>

            <div className="mt-4 min-h-6 text-sm text-[color:var(--ink-muted)]">{ecMessage}</div>
            {ecCandidates.length > 1 ? (
              <div className="mt-3 rounded-xl border border-[color:var(--line)] bg-white/80 p-3 text-sm">
                <div className="mb-2 font-medium">候補を選択</div>
                <div className="grid gap-2">
                  {ecCandidates.map((candidate) => (
                    <button
                      key={`${candidate.zip_code}-${candidate.city_id}-${candidate.town}`}
                      type="button"
                      className="ghost-button justify-start"
                      onClick={() => setEcForm((prev) => applyAddressFromRecord(prev, candidate))}
                    >
                      <MapPinned className="h-4 w-4" />
                      {candidate.prefecture}
                      {candidate.city}
                      {candidate.town} ({formatZip(candidate.zip_code)})
                    </button>
                  ))}
                </div>
              </div>
            ) : null}
          </motion.section>
        ) : (
          <motion.section key="member-panel" {...panelAnimation} className="mt-6 surface-card p-6 md:p-8">
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
                  onChange={(event) => setMemberForm((prev) => ({ ...prev, fullName: event.target.value }))}
                  placeholder="郵便 花子"
                />
              </label>

              <label className="field-block">
                <span className="field-label">メールアドレス</span>
                <input
                  className="outline-input"
                  value={memberForm.email}
                  onChange={(event) => setMemberForm((prev) => ({ ...prev, email: event.target.value }))}
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
                  <button type="button" className="pill-button" onClick={handleMemberZipLookup} disabled={memberLoading}>
                    <Search className="h-4 w-4" />
                    郵便番号で補完
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
                    <Building2 className="h-4 w-4" />
                    キーワード検索
                  </button>
                </div>
              </label>

              <label className="field-block">
                <span className="field-label">都道府県</span>
                <input
                  className="outline-input"
                  value={memberForm.prefecture}
                  onChange={(event) => setMemberForm((prev) => ({ ...prev, prefecture: event.target.value }))}
                />
              </label>

              <label className="field-block">
                <span className="field-label">市区町村</span>
                <input
                  className="outline-input"
                  value={memberForm.city}
                  onChange={(event) => setMemberForm((prev) => ({ ...prev, city: event.target.value }))}
                />
              </label>

              <label className="field-block">
                <span className="field-label">町域</span>
                <input
                  className="outline-input"
                  value={memberForm.town}
                  onChange={(event) => setMemberForm((prev) => ({ ...prev, town: event.target.value }))}
                />
              </label>

              <label className="field-block">
                <span className="field-label">番地・建物名</span>
                <input
                  className="outline-input"
                  value={memberForm.addressDetail}
                  onChange={(event) => setMemberForm((prev) => ({ ...prev, addressDetail: event.target.value }))}
                  placeholder="2-8-1"
                />
              </label>
            </div>

            <div className="mt-4 min-h-6 text-sm text-[color:var(--ink-muted)]">{memberMessage}</div>
            {memberCandidates.length > 0 ? (
              <div className="mt-3 rounded-xl border border-[color:var(--line)] bg-white/80 p-3 text-sm">
                <div className="mb-2 font-medium">検索候補</div>
                <div className="grid gap-2">
                  {memberCandidates.map((candidate) => (
                    <button
                      key={`member-${candidate.zip_code}-${candidate.city_id}-${candidate.town}`}
                      type="button"
                      className="ghost-button justify-start"
                      onClick={() => setMemberForm((prev) => applyAddressFromRecord(prev, candidate))}
                    >
                      <MapPinned className="h-4 w-4" />
                      {candidate.prefecture}
                      {candidate.city}
                      {candidate.town} ({formatZip(candidate.zip_code)})
                    </button>
                  ))}
                </div>
              </div>
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
          フォーム側はSDKの `lookupZip` / `searchAddress` を呼ぶだけで導入できます。
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
