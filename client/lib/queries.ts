import { gql } from "graphql-request";

export const PLACE_ORDER = gql`
  mutation PlaceOrder(
    $userId: ID!
    $symbol: SymbolGql!
    $side: SideGql!
    $orderType: OrderTypeGql!
    $quantity: String!
    $price: String
  ) {
    placeOrder(
      userId: $userId
      symbol: $symbol
      side: $side
      orderType: $orderType
      quantity: $quantity
      price: $price
    ) {
      id
      symbol
      side
      orderType
      price
      quantity
      filledQuantity
      status
    }
  }
`;

export const GET_PORTFOLIO = gql`
  query GetPortfolio($userId: ID!) {
    portfolio(userId: $userId) {
      cashBalance
      totalMarketValue
      totalUnrealizedPnl
      positions {
        symbol
        quantity
        avgCost
        marketValue
        unrealizedPnl
      }
    }
  }
`;

export const GET_ORDER_HISTORY = gql`
  query GetOrderHistory($userId: ID!) {
    orderHistory(userId: $userId) {
      id
      symbol
      side
      orderType
      price
      quantity
      filledQuantity
      status
    }
  }
`;

export const GET_TRADE_HISTORY = gql`
  query GetTradeHistory($userId: ID!) {
    tradeHistory(userId: $userId) {
      id
      symbol
      price
      quantity
      executedAt
    }
  }
`;

export const GET_ALERTS = gql`
  query GetAlerts($userId: ID!) {
    alerts(userId: $userId) {
      id
      symbol
      targetPrice
      direction
      triggered
    }
  }
`;

export const CREATE_ALERT = gql`
  mutation CreateAlert($userId: ID!, $symbol: SymbolGql!, $targetPrice: String!, $direction: AlertDirectionGql!) {
    createAlert(userId: $userId, symbol: $symbol, targetPrice: $targetPrice, direction: $direction) {
      id
      symbol
      targetPrice
      direction
      triggered
    }
  }
`;

export const LOGIN = gql`
  mutation Login($username: String!, $password: String!) {
    login(username: $username, password: $password) {
      token
      account {
        id
        username
        cashBalance
      }
    }
  }
`;

export const REGISTER = gql`
  mutation Register($username: String!, $password: String!) {
    register(username: $username, password: $password) {
      token
      account {
        id
        username
        cashBalance
      }
    }
  }
`;

export const LOGOUT = gql`
  mutation Logout($token: String!) {
    logout(token: $token)
  }
`;

export const GET_CANDLES = gql`
  query GetCandles($symbol: SymbolGql!, $limit: Int!) {
    candles(symbol: $symbol, limit: $limit) {
      bucketStart
      open
      high
      low
      close
    }
  }
`;

export const DEPOSIT = gql`
  mutation Deposit($userId: ID!, $amount: String!) {
    deposit(userId: $userId, amount: $amount) {
      cashBalance
    }
  }
`;