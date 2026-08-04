"use client";

import { useEffect, useRef } from "react";
import { createChart, ColorType, CandlestickSeries, IChartApi, ISeriesApi } from "lightweight-charts";
import { useCandles } from "@/lib/use-candles";
import { useMarketPrice } from "@/lib/use-market-price";
import { useTheme } from "@/app/providers";

export function CandlestickChart({ symbol }: { symbol: string }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const seriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const { candles, loading } = useCandles(symbol);
  const liveTick = useMarketPrice(symbol);
  const { theme } = useTheme();

  // Initialize chart once
  useEffect(() => {
    if (!containerRef.current) return;

    const isDark = theme === "dark";
    const chart = createChart(containerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: isDark ? "#9ca3af" : "#374151",
      },
      grid: {
        vertLines: { color: isDark ? "#1f2937" : "#e5e7eb" },
        horzLines: { color: isDark ? "#1f2937" : "#e5e7eb" },
      },
      width: containerRef.current.clientWidth,
      height: 360,
      timeScale: { timeVisible: true, secondsVisible: false },
    });

    const series = chart.addSeries(CandlestickSeries, {
      upColor: "#22c55e",
      downColor: "#ef4444",
      borderVisible: false,
      wickUpColor: "#22c55e",
      wickDownColor: "#ef4444",
    });

    chartRef.current = chart;
    seriesRef.current = series;

    const handleResize = () => {
      if (containerRef.current) {
        chart.applyOptions({ width: containerRef.current.clientWidth });
      }
    };
    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      chart.remove();
      chartRef.current = null;
      seriesRef.current = null;
    };
  }, [theme]);

  // Load historical candles
  useEffect(() => {
    if (!seriesRef.current || loading || candles.length === 0) return;

    const formatted = candles.map((c) => ({
      time: (new Date(c.bucketStart).getTime() / 1000) as any,
      open: parseFloat(c.open),
      high: parseFloat(c.high),
      low: parseFloat(c.low),
      close: parseFloat(c.close),
    }));

    seriesRef.current.setData(formatted);
    chartRef.current?.timeScale().fitContent();
  }, [candles, loading]);

  // Update the current (rightmost) candle live as ticks stream in
  useEffect(() => {
    if (!seriesRef.current || !liveTick) return;

    const price = parseFloat(liveTick.price);
    const now = Math.floor(Date.now() / 1000);
    const bucketTime = (Math.floor(now / 60) * 60) as any;

    seriesRef.current.update({
      time: bucketTime,
      open: price,
      high: price,
      low: price,
      close: price,
    });
  }, [liveTick]);

  return (
    <div className="rounded-lg border border-gray-200 dark:border-gray-800 p-4 bg-white dark:bg-gray-900">
      <div ref={containerRef} className="w-full" />
      {loading && <p className="text-gray-500 dark:text-gray-400 text-sm mt-2">Loading chart...</p>}
    </div>
  );
}