"use client";

import * as Form from "@radix-ui/react-form";
import React, { useEffect, useState } from "react";

// Define types for API responses
type Prefecture = {
  prefecture_id: number;
  prefecture: string;
};

type City = {
  city_id: string;
  city: string;
};

type PostalCodeData = {
  zip_code: string;
  prefecture_id: number;
  city_id: string;
  prefecture: string;
  city: string;
  town: string;
};

const FormDemo: React.FC = () => {
  // State for dynamic data
  const [prefectures, setPrefectures] = useState<Prefecture[]>([]);
  const [cities, setCities] = useState<City[]>([]);

  // Form state
  const [postalCode1, setPostalCode1] = useState("");
  const [postalCode2, setPostalCode2] = useState("");
  const [selectedPrefectureId, setSelectedPrefectureId] = useState<string>("");
  const [selectedCityId, setSelectedCityId] = useState<string>("");
  const [town, setTown] = useState("");
  const [otherAddress, setOtherAddress] = useState("");

  // Fetch prefectures on mount
  useEffect(() => {
    const fetchPrefectures = async () => {
      try {
        const res = await fetch(
          `${process.env.NEXT_PUBLIC_API_URL}/postal_codes/prefectures`
        );
        if (res.ok) {
          const data = await res.json();
          setPrefectures(data);
        }
      } catch (error) {
        console.error("Failed to fetch prefectures:", error);
      }
    };
    fetchPrefectures();
  }, []);

  // Fetch cities when prefecture changes
  useEffect(() => {
    const fetchCities = async () => {
      if (!selectedPrefectureId) {
        setCities([]);
        return;
      }
      try {
        const res = await fetch(
          `${process.env.NEXT_PUBLIC_API_URL}/postal_codes/cities?prefecture_id=${selectedPrefectureId}`
        );
        if (res.ok) {
          const data = await res.json();
          setCities(data);
        }
      } catch (error) {
        console.error("Failed to fetch cities:", error);
      }
    };
    fetchCities();
  }, [selectedPrefectureId]);

  // Handle postal code search
  const handlePostalCodeSearch = async () => {
    const zip = `${postalCode1}${postalCode2}`;
    if (zip.length !== 7) return;

    try {
      const res = await fetch(
        `${process.env.NEXT_PUBLIC_API_URL}/postal_codes/${zip}`
      );
      if (res.ok) {
        const data: PostalCodeData[] = await res.json();
        if (data.length > 0) {
          const result = data[0];
          setSelectedPrefectureId(result.prefecture_id.toString());
          // We need to wait for cities to update before setting city_id,
          // or we can just set it and rely on the select value matching.
          // However, since cities fetch is async via useEffect, we might have a race condition.
          // For simplicity in this demo, we'll set it and let React handle the update cycle,
          // but strictly speaking we might want to fetch cities immediately here too.
          // Let's try setting it directly.

          // Trigger city fetch manually to ensure we have options to select from
          const citiesRes = await fetch(
            `${process.env.NEXT_PUBLIC_API_URL}/postal_codes/cities?prefecture_id=${result.prefecture_id}`
          );
          if (citiesRes.ok) {
            const citiesData = await citiesRes.json();
            setCities(citiesData);
            setSelectedCityId(result.city_id);
          }

          setTown(result.town);
        }
      } else {
        alert("郵便番号が見つかりませんでした。");
      }
    } catch (error) {
      console.error("Failed to search postal code:", error);
      alert("検索中にエラーが発生しました。");
    }
  };

  return (
    <div className="border-slate-600 dark:border-slate-50 border-[1px] w-1/3">
      <Form.Root className="border-1">
        <Form.Field name="postalCode" className="p-4 flex-row">
          <Form.Label className="font-bold text-[15px]">郵便番号</Form.Label>
          <div className="flex gap-4 mt-2 items-center">
            <input
              className="p-1 border-slate-600 dark:border-slate-50 border-[1px] rounded-[4px] w-[60px] h-[30px] text-right"
              type="text"
              maxLength={3}
              placeholder="123"
              value={postalCode1}
              onChange={(e) => setPostalCode1(e.target.value)}
            />
            -
            <input
              className="p-1 border-slate-600 dark:border-slate-50 border-[1px] rounded-[4px] w-[70px] h-[30px] text-right"
              type="text"
              maxLength={4}
              placeholder="4567"
              value={postalCode2}
              onChange={(e) => setPostalCode2(e.target.value)}
            />
            <button
              type="button"
              onClick={handlePostalCodeSearch}
              className="ml-2 px-3 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
            >
              住所検索
            </button>
          </div>
        </Form.Field>

        <Form.Field name="prefecture" className="p-4 gap-4">
          <Form.Label className="font-bold text-[15px]">都道府県</Form.Label>
          <br />
          <select
            className="mt-2 p-1 border-[1px] border-slate-600 dark:border-slate-50 rounded-[4px] w-full"
            value={selectedPrefectureId}
            onChange={(e) => {
              console.log(e.target.value);
              setSelectedPrefectureId(e.target.value);
              setSelectedCityId("");
              setTown("");
            }}
          >
            <option value="">選択してください</option>
            {prefectures.map((pref) => (
              <option
                key={pref.prefecture_id}
                value={pref.prefecture_id.toString()}
              >
                {pref.prefecture}
              </option>
            ))}
          </select>
        </Form.Field>

        <Form.Field name="city" className="p-4 gap-4">
          <Form.Label className="font-bold text-[14px]">市区町村</Form.Label>
          <br />
          <select
            name="city"
            className="mt-2 p-1 border-[1px] border-slate-600 dark:border-slate-50 rounded-[4px] w-full"
            value={selectedCityId}
            onChange={(e) => {
              setSelectedCityId(e.target.value);
              setTown("");
            }}
            disabled={!selectedPrefectureId}
          >
            <option value="">選択してください</option>
            {cities.map((city) => (
              <option key={city.city_id} value={city.city_id}>
                {city.city}
              </option>
            ))}
          </select>
        </Form.Field>

        <Form.Field name="town" className="p-4 gap-4">
          <Form.Label className="font-bold text-[15px]">町域</Form.Label>
          <br />
          <input
            type="text"
            className="p-1 mt-2 border-slate-600 dark:border-slate-50 border-[1px] rounded-[4px] h-8 w-full"
            value={town}
            onChange={(e) => setTown(e.target.value)}
          />
        </Form.Field>

        <Form.Field name="other" className="p-4 gap-4">
          <Form.Label className="font-bold text-[15px]">
            番地・建物名など
          </Form.Label>
          <br />
          <input
            type="text"
            className="p-1 mt-2 border-slate-600 dark:border-slate-50 border-[1px] rounded-[4px] h-8 w-full"
            value={otherAddress}
            onChange={(e) => setOtherAddress(e.target.value)}
          />
        </Form.Field>

        <div className="flex justify-center gap-4 p-4">
          <Form.Submit asChild>
            <button
              type="submit"
              className="mt-2 font-bold text-[16px] bg-slate-200 dark:text-slate-600 dark:bg-slate-50 border-[1px] rounded-[4px] w-32 h-10"
              onClick={(e) => {
                e.preventDefault();
                alert(
                  `送信内容:\n郵便番号: ${postalCode1}-${postalCode2}\n都道府県ID: ${selectedPrefectureId}\n市区町村ID: ${selectedCityId}\n町域: ${town}\nその他: ${otherAddress}`
                );
              }}
            >
              登録
            </button>
          </Form.Submit>
        </div>
      </Form.Root>
    </div>
  );
};

export default FormDemo;
