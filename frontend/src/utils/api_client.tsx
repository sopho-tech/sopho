export const ApiService = {
  post: async ({
    url,
    data,
    headers,
    credentials = true,
  }: {
    url: string;
    data?: any;
    headers?: Record<string, string>;
    credentials?: boolean;
  }) => {
    const response = await fetch(url, {
      headers: headers,
      method: "POST",
      body: data ? JSON.stringify(data) : undefined,
      credentials: credentials ? "include" : undefined,
    });
    if (!response.ok) {
      throw new Error(`Failed to post: ${response.statusText}`);
    }
    return response.json();
  },
  get: async <T = any,>({
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
    if (onlyBody) {
      return response.json() as Promise<T>;
    }
    return response;
  },
  put: async ({
    url,
    data,
    headers,
    credentials = true,
  }: {
    url: string;
    data?: any;
    headers?: Record<string, string>;
    credentials?: boolean;
  }) => {
    const response = await fetch(url, {
      headers: headers,
      method: "PUT",
      body: data ? JSON.stringify(data) : undefined,
      credentials: credentials ? "include" : undefined,
    });
    return response.json();
  },
};
