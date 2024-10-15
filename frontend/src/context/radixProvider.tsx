"use client";

import React, { ReactNode, useEffect, useState } from "react";
import { Theme } from "@radix-ui/themes";
import SwitchSample from "@/components/switch";

interface RadixProviderProps {
  children: ReactNode;
  isDarkMode?: boolean;
}

const RadixProvider: React.FC<RadixProviderProps> = ({ children }) => {
  const [isDarkTheme, setIsDarkTheme] = useState(false);

  useEffect(() => {
    // ダークモードならhtmlに'dark'クラスを追加
    document.documentElement.classList.toggle("dark", isDarkTheme);
  }, [isDarkTheme]);

  const toggleTheme = () => {
    setIsDarkTheme(!isDarkTheme);
  };

  return (
    <Theme
      appearance={isDarkTheme ? "dark" : "light"}
      accentColor="crimson"
      grayColor="slate"
      radius="large"
      scaling="95%"
      className="w-full h-full"
    >
      <div style={{ position: "absolute", right: "1px" }}>
        <SwitchSample label="Theme" onSwitch={toggleTheme} />
      </div>
      {children}
    </Theme>
  );
};

export default RadixProvider;
