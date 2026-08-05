export type AssetCategory = "Crypto" | "Stocks";

export type Asset = {
  gqlValue: string;   // matches backend SymbolGql enum variant
  wsSymbol: string;   // matches WebSocket channel / GraphQL Symbol string
  label: string;
  category: AssetCategory;
};

export const ASSETS: Asset[] = [
  { gqlValue: "BTC_USD", wsSymbol: "BTC-USD", label: "BTC/USD", category: "Crypto" },
  { gqlValue: "ETH_USD", wsSymbol: "ETH-USD", label: "ETH/USD", category: "Crypto" },
  { gqlValue: "AAPL", wsSymbol: "AAPL", label: "Apple (AAPL)", category: "Stocks" },
  { gqlValue: "TSLA", wsSymbol: "TSLA", label: "Tesla (TSLA)", category: "Stocks" },
  { gqlValue: "GOOGL", wsSymbol: "GOOGL", label: "Alphabet (GOOGL)", category: "Stocks" },
  { gqlValue: "MSFT", wsSymbol: "MSFT", label: "Microsoft (MSFT)", category: "Stocks" },
  { gqlValue: "AMZN", wsSymbol: "AMZN", label: "Amazon (AMZN)", category: "Stocks" },
];

export function assetByWsSymbol(wsSymbol: string): Asset | undefined {
  return ASSETS.find((a) => a.wsSymbol === wsSymbol);
}

export function assetByGqlValue(gqlValue: string): Asset | undefined {
  return ASSETS.find((a) => a.gqlValue === gqlValue);
}