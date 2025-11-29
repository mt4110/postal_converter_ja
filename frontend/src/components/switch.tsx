"use client";

import React from "react";
import * as Switch from "@radix-ui/react-switch";

type Props = {
  label: string;
  className?: string;
  onSwitch: () => void;
};

const SwitchSample: React.FC<Props> = (props: Props) => (
  <div
    className={`${props.className ? props.className : ""} flex items-center`}
  >
    <label
      className="pr-[15px] text-[15px] leading-none text-slate-black dark:text-slate-50"
      htmlFor="airplane-mode"
    >
      {props.label}
    </label>
    <Switch.Root
      className="relative h-[25px] w-[42px] cursor-default rounded-full bg-blackA6 shadow-[0_2px_10px] shadow-blackA4 outline-none focus:shadow-[0_0_0_0px] focus:shadow-black data-[state=checked]:bg-black"
      id="airplane-mode"
      onClick={props.onSwitch}
    >
      <Switch.Thumb className="block size-[21px] translate-x-0.5 rounded-full bg-white shadow-[0_2px_2px] shadow-blackA4 transition-transform duration-100 will-change-transform data-[state=checked]:translate-x-[19px]" />
    </Switch.Root>
  </div>
);

export default SwitchSample;
