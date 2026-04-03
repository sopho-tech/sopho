export type FetchSseOptions = {
  signal?: AbortSignal;
  credentials?: RequestCredentials;
  method?: "GET" | "POST";
};

export type FetchSseCallbacks = {
  onMessage: (data: string) => void;
};

function parseSseEventBlock(raw: string): string | null {
  const lines = raw.split("\n");
  const dataLines: string[] = [];
  for (const line of lines) {
    if (line.startsWith(":")) continue;
    if (line.startsWith("data:")) {
      let rest = line.slice(5);
      if (rest.startsWith(" ")) rest = rest.slice(1);
      dataLines.push(rest);
    }
  }
  if (dataLines.length === 0) return null;
  return dataLines.join("\n");
}

function drainSseBuffer(
  buffer: string,
  onMessage: (data: string) => void,
): string {
  const sep = "\n\n";
  let idx = buffer.indexOf(sep);
  while (idx !== -1) {
    const raw = buffer.slice(0, idx);
    buffer = buffer.slice(idx + sep.length);
    const data = parseSseEventBlock(raw);
    if (data !== null) onMessage(data);
    idx = buffer.indexOf(sep);
  }
  return buffer;
}

export async function streamSse(
  url: string,
  options: FetchSseOptions,
  callbacks: FetchSseCallbacks,
): Promise<void> {
  const { signal, credentials = "include", method = "GET" } = options;
  let response: Response;
  try {
    response = await fetch(url, {
      method,
      credentials,
      signal,
      headers: {
        Accept: "text/event-stream",
      },
    });
  } catch (e) {
    if (e instanceof DOMException && e.name === "AbortError") return;
    throw e instanceof Error ? e : new Error(String(e));
  }
  if (!response.ok) {
    let detail = "";
    try {
      const j = (await response.json()) as { error?: string };
      detail = typeof j?.error === "string" ? j.error : JSON.stringify(j);
    } catch {
      try {
        detail = await response.text();
      } catch {
        detail = "";
      }
    }
    throw new Error(detail || `HTTP ${response.status}`);
  }
  const reader = response.body?.getReader();
  if (!reader) throw new Error("No response body");
  const decoder = new TextDecoder();
  let buffer = "";
  try {
    while (true) {
      let readResult: ReadableStreamReadResult<Uint8Array>;
      try {
        readResult = await reader.read();
      } catch (e) {
        if (e instanceof DOMException && e.name === "AbortError") return;
        throw e instanceof Error ? e : new Error(String(e));
      }
      const { done, value } = readResult;
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      buffer = buffer.replace(/\r\n/g, "\n");
      buffer = drainSseBuffer(buffer, callbacks.onMessage);
    }
    if (buffer.trim()) {
      drainSseBuffer(`${buffer}\n\n`, callbacks.onMessage);
    }
  } finally {
    reader.releaseLock();
  }
}
