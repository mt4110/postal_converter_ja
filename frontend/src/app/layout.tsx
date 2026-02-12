import type { Metadata } from "next";
import { Noto_Sans_JP, Space_Grotesk } from "next/font/google";
import "./globals.css";

export const metadata: Metadata = {
  title: "Postal Converter JA | SDK Samples",
  description: "ECフォーム・会員登録フォーム向け郵便番号住所補完サンプル",
};

const displayFont = Space_Grotesk({
  subsets: ["latin"],
  variable: "--font-display",
  weight: ["500", "700"],
});

const textFont = Noto_Sans_JP({
  subsets: ["latin"],
  variable: "--font-text",
  weight: ["400", "500", "700"],
});

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="ja">
      <body className={`${displayFont.variable} ${textFont.variable} app-body antialiased`}>
        {children}
      </body>
    </html>
  );
}
