"use client";

import * as Form from "@radix-ui/react-form";
import React from "react";
import prefJson from "../../../../worker/common/pref.json";

const FormDemo: React.FC = () => {
  // 郵便番号検索フォーム(実際に利用する場合は、アーキテクチャや決まり事に則りコンポーネントを分けたりしてください。)
  return (
    <div className="border-slate-600 dark:border-slate-50 border-[1px] w-1/4">
      <Form.Root className="border-1">
        <Form.Field name="postalCode" className="p-4 flex-row">
          <Form.Label className="font-bold text-[15px]">郵便番号</Form.Label>
          <div className="flex gap-4 mt-2">
            <Form.Control asChild>
              <input
                className="p-1 border-slate-600 dark:border-slate-50 border-[1px] rounded-[4px] w-[60px] h-[30px] text-right"
                type="number"
                maxLength={3}
                placeholder="123"
              />
            </Form.Control>
            -
            <Form.Control asChild>
              <input
                className="p-1 border-slate-600 dark:border-slate-50 border-[1px] rounded-[4px] w-[70px] h-[30px] text-right"
                type="number"
                maxLength={4}
                placeholder="4567"
              />
            </Form.Control>
          </div>
        </Form.Field>

        <Form.Field name="prefecture" className="p-4 gap-4">
          <Form.Label className="font-bold text-[15px]">都道府県</Form.Label>
          <br />
          <Form.Control asChild>
            <select className="mt-2 p-1 border-[1px] border-slate-600 dark:border-slate-50 rounded-[4px]">
              {prefJson.map((pref) => (
                <option value={pref.id}>{pref.label}</option>
              ))}
            </select>
          </Form.Control>
        </Form.Field>

        <Form.Field name="city" className="p-4 gap-4">
          <Form.Label className="font-bold text-[14px]">市区町村</Form.Label>
          <br />
          <Form.Control asChild>
            <input
              type="text"
              className="p-1 mt-2 border-slate-600 dark:border-slate-50 border-[1px] rounded-[4px] h-8 w-60"
            />
          </Form.Control>
        </Form.Field>

        <Form.Field name="other" className="p-4 gap-4">
          <Form.Label className="font-bold text-[15px]">その他</Form.Label>
          <br />
          <Form.Control asChild>
            <input
              type="text"
              className="p-1 mt-2 border-slate-600 dark:border-slate-50 border-[1px] rounded-[4px] h-8 w-60"
            />
          </Form.Control>
        </Form.Field>

        <div className="flex justify-center gap-4 p-4">
          <Form.Submit asChild>
            <button
              type="submit"
              className="mt-2 font-bold text-[16px] bg-slate-200 dark:text-slate-600 dark:bg-slate-50 border-[1px] rounded-[4px] w-32 h-10"
            >
              検索
            </button>
          </Form.Submit>
        </div>
      </Form.Root>
    </div>
  );
};

export default FormDemo;
