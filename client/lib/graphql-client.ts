import { GraphQLClient } from "graphql-request";
import { getStoredToken } from "./session";

const endpoint = process.env.NEXT_PUBLIC_GRAPHQL_URL ?? "http://localhost:8000/graphql";

export const gqlClient = new GraphQLClient(endpoint, {
  requestMiddleware: (request) => {
    const token = getStoredToken();
    return {
      ...request,
      headers: {
        ...request.headers,
        ...(token ? { "X-Session-Token": token } : {}),
      },
    };
  },
});