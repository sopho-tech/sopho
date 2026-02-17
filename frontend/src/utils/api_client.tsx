export const ApiService = {
  post: async <T = unknown,>({
    url,
    data,
    headers,
    credentials = true,
    onlyBody = true,
  }: {
    url: string;
    data?: unknown;
    headers?: Record<string, string>;
    credentials?: boolean;
    onlyBody?: boolean;
  }) => {
    const response = await fetch(url, {
      headers: headers,
      method: "POST",
      body: data ? JSON.stringify(data) : undefined,
      credentials: credentials ? "include" : undefined,
    });
    if (!response.ok) {
      throw response;
    }
    if (onlyBody) {
      return response.json() as Promise<T>;
    }
    return response;
  },
  get: async <T = unknown,>({
    url,
    headers,
    credentials = true,
    onlyBody = false,
  }: {
    url: string;
    headers?: Record<string, string>;
    credentials?: boolean;
    onlyBody?: boolean;
  }): Promise<T | Response> => {
    const response = await fetch(url, {
      headers: headers,
      method: "GET",
      credentials: credentials ? "include" : undefined,
    });
    if (!response.ok) {
      throw response;
    }
    if (onlyBody) {
      return response.json() as Promise<T>;
    }
    return response;
  },
  put: async <T = unknown,>({
    url,
    data,
    headers,
    credentials = true,
    onlyBody = true,
  }: {
    url: string;
    data?: unknown;
    headers?: Record<string, string>;
    credentials?: boolean;
    onlyBody?: boolean;
  }) => {
    const response = await fetch(url, {
      headers: headers,
      method: "PUT",
      body: data ? JSON.stringify(data) : undefined,
      credentials: credentials ? "include" : undefined,
    });
    if (!response.ok) {
      throw response;
    }
    if (onlyBody) {
      return response.json() as Promise<T>;
    }
    return response;
  },
  patch: async <T = unknown,>({
    url,
    data,
    headers,
    credentials = true,
    onlyBody = true,
  }: {
    url: string;
    data?: unknown;
    headers?: Record<string, string>;
    credentials?: boolean;
    onlyBody?: boolean;
  }) => {
    const response = await fetch(url, {
      headers: headers,
      method: "PATCH",
      body: data ? JSON.stringify(data) : undefined,
      credentials: credentials ? "include" : undefined,
    });
    if (!response.ok) {
      throw response;
    }
    if (onlyBody) {
      return response.json() as Promise<T>;
    }
    return response;
  },
};
