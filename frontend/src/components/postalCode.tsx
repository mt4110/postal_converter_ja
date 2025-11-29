"use client";

import * as Form from "@radix-ui/react-form";
import React, { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { Search, MapPin, Send, Building2, Home } from "lucide-react";

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
  const [isSearching, setIsSearching] = useState(false);

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

    setIsSearching(true);
    try {
      const res = await fetch(
        `${process.env.NEXT_PUBLIC_API_URL}/postal_codes/${zip}`
      );
      if (res.ok) {
        const data: PostalCodeData[] = await res.json();
        if (data.length > 0) {
          const result = data[0];
          setSelectedPrefectureId(result.prefecture_id.toString());

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
    } finally {
      setIsSearching(false);
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      className="glass-card rounded-2xl p-8 w-full max-w-md mx-auto text-white"
    >
      <div className="mb-8 text-center">
        <h2 className="text-2xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-200 to-purple-200">
          Address Registration
        </h2>
        <p className="text-white/60 text-sm mt-2">郵便番号から住所を自動入力</p>
      </div>

      <Form.Root className="space-y-6">
        <Form.Field name="postalCode">
          <div className="flex justify-between items-baseline mb-2">
            <Form.Label className="font-medium text-sm text-white/80 flex items-center gap-2">
              <Search className="w-4 h-4" /> 郵便番号
            </Form.Label>
            <span className="text-xs text-white/40">Required</span>
          </div>
          <div className="flex gap-2 items-center">
            <input
              className="glass p-2 rounded-lg w-20 text-center focus:outline-none focus:ring-2 focus:ring-blue-400/50 transition-all"
              type="text"
              maxLength={3}
              placeholder="123"
              value={postalCode1}
              onChange={(e) => setPostalCode1(e.target.value)}
            />
            <span className="text-white/40">-</span>
            <input
              className="glass p-2 rounded-lg w-24 text-center focus:outline-none focus:ring-2 focus:ring-blue-400/50 transition-all"
              type="text"
              maxLength={4}
              placeholder="4567"
              value={postalCode2}
              onChange={(e) => setPostalCode2(e.target.value)}
            />
            <motion.button
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              type="button"
              onClick={handlePostalCodeSearch}
              disabled={isSearching}
              className="ml-auto px-4 py-2 text-sm bg-blue-500/80 hover:bg-blue-500 text-white rounded-lg shadow-lg shadow-blue-500/20 transition-all flex items-center gap-2 disabled:opacity-50"
            >
              {isSearching ? (
                <motion.div
                  animate={{ rotate: 360 }}
                  transition={{ repeat: Infinity, duration: 1, ease: "linear" }}
                >
                  <Search className="w-4 h-4" />
                </motion.div>
              ) : (
                <>
                  <Search className="w-4 h-4" /> 検索
                </>
              )}
            </motion.button>
          </div>
        </Form.Field>

        <div className="grid grid-cols-2 gap-4">
          <Form.Field name="prefecture">
            <Form.Label className="font-medium text-sm text-white/80 mb-2 block flex items-center gap-2">
              <MapPin className="w-4 h-4" /> 都道府県
            </Form.Label>
            <select
              className="glass w-full p-2 rounded-lg text-white/90 focus:outline-none focus:ring-2 focus:ring-blue-400/50 appearance-none cursor-pointer"
              value={selectedPrefectureId}
              onChange={(e) => {
                setSelectedPrefectureId(e.target.value);
                setSelectedCityId("");
                setTown("");
              }}
            >
              <option value="" className="text-black">
                選択してください
              </option>
              {prefectures.map((pref) => (
                <option
                  key={pref.prefecture_id}
                  value={pref.prefecture_id.toString()}
                  className="text-black"
                >
                  {pref.prefecture}
                </option>
              ))}
            </select>
          </Form.Field>

          <Form.Field name="city">
            <Form.Label className="font-medium text-sm text-white/80 mb-2 block flex items-center gap-2">
              <Building2 className="w-4 h-4" /> 市区町村
            </Form.Label>
            <select
              name="city"
              className="glass w-full p-2 rounded-lg text-white/90 focus:outline-none focus:ring-2 focus:ring-blue-400/50 appearance-none cursor-pointer disabled:opacity-50"
              value={selectedCityId}
              onChange={(e) => {
                setSelectedCityId(e.target.value);
                setTown("");
              }}
              disabled={!selectedPrefectureId}
            >
              <option value="" className="text-black">
                選択してください
              </option>
              {cities.map((city) => (
                <option
                  key={city.city_id}
                  value={city.city_id}
                  className="text-black"
                >
                  {city.city}
                </option>
              ))}
            </select>
          </Form.Field>
        </div>

        <Form.Field name="town">
          <Form.Label className="font-medium text-sm text-white/80 mb-2 block flex items-center gap-2">
            <Home className="w-4 h-4" /> 町域
          </Form.Label>
          <input
            type="text"
            className="glass w-full p-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-400/50 transition-all"
            value={town}
            onChange={(e) => setTown(e.target.value)}
          />
        </Form.Field>

        <Form.Field name="other">
          <Form.Label className="font-medium text-sm text-white/80 mb-2 block">
            番地・建物名など
          </Form.Label>
          <input
            type="text"
            className="glass w-full p-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-400/50 transition-all"
            value={otherAddress}
            onChange={(e) => setOtherAddress(e.target.value)}
          />
        </Form.Field>

        <div className="pt-4">
          <Form.Submit asChild>
            <motion.button
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
              type="submit"
              className="w-full py-3 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white font-bold rounded-xl shadow-lg shadow-purple-500/30 transition-all flex justify-center items-center gap-2"
              onClick={(e) => {
                e.preventDefault();
                alert(
                  `送信内容:\n郵便番号: ${postalCode1}-${postalCode2}\n都道府県ID: ${selectedPrefectureId}\n市区町村ID: ${selectedCityId}\n町域: ${town}\nその他: ${otherAddress}`
                );
              }}
            >
              <Send className="w-4 h-4" /> 登録する
            </motion.button>
          </Form.Submit>
        </div>
      </Form.Root>
    </motion.div>
  );
};

export default FormDemo;
