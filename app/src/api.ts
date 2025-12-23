import { Configuration, DefaultApi } from "./backend-client/index"

export function createRetriever<T>(f: (api: DefaultApi) => T): (jwt: string) => T {
  return (jwt) => { 
    const config = new Configuration({
      basePath: import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080/api",
      accessToken: () => jwt
    });

    const api = new DefaultApi(config);
    const selected = f(api);

    return selected
  }
} 
