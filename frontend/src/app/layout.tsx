import type { Metadata } from "next";
import "./globals.css";
import "@radix-ui/themes/styles.css";
import RadixProvider from "@/context/radixProvider";
import { Inter } from "next/font/google";

const metadata: Metadata = {
  title: "Sample Postal form code",
  description: "postal code japan form",
};

const inter = Inter({
  weight: ["100", "400"],
  style: "normal",
  subsets: ["latin"],
  variable: "--font-inter",
});

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <meta name="title" content={metadata.description!} />
        <meta name="description" content={metadata.description!} />

        <link rel="icon" href="/favicon.ico" />
      </head>

      <body
        className={`${inter.variable} antialiased flex items-center justify-center w-full`}
      >
        <RadixProvider>{children}</RadixProvider>
      </body>
    </html>
  );
}
