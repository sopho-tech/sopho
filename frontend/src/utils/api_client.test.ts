import { afterEach, describe, expect, test, vi } from "vitest";
import { ApiError } from "src/api/dto";
import { ApiService } from "src/utils/api_client";

const REQUEST_ID = "3f1c2a9e-5b7d-4c1a-9e2f-0a1b2c3d4e5f";
const ANY_URL = "/api/v1/anything";

function respondWith(response: Response) {
  vi.stubGlobal(
    "fetch",
    vi.fn(() => Promise.resolve(response)),
  );
}

async function failedRequest(): Promise<ApiError> {
  try {
    await ApiService.get({ url: ANY_URL, onlyBody: true });
  } catch (error) {
    if (error instanceof ApiError) {
      return error;
    }
    throw error;
  }
  throw new Error("the request did not fail");
}

describe("API client errors", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test("takes the request id from the response header of a JSON error", async () => {
    respondWith(
      new Response(JSON.stringify({ error: "Database error" }), {
        status: 500,
        headers: {
          "content-type": "application/json",
          "x-request-id": REQUEST_ID,
        },
      }),
    );

    const error = await failedRequest();

    expect(error.status).toBe(500);
    expect(error.requestId).toBe(REQUEST_ID);
    expect(error.message).toBe(`Database error (Request ID: ${REQUEST_ID})`);
  });

  test("takes the request id from the header of an error that is not JSON", async () => {
    respondWith(
      new Response("Failed to deserialize the JSON body", {
        status: 422,
        headers: { "content-type": "text/plain", "x-request-id": REQUEST_ID },
      }),
    );

    const error = await failedRequest();

    expect(error.status).toBe(422);
    expect(error.requestId).toBe(REQUEST_ID);
    expect(error.message).toBe("Request failed with 422");
  });

  test("has no request id when the response carries none", async () => {
    respondWith(
      new Response(JSON.stringify({ message: "Tenant not found" }), {
        status: 404,
        headers: { "content-type": "application/json" },
      }),
    );

    const error = await failedRequest();

    expect(error.requestId).toBeUndefined();
    expect(error.message).toBe("Tenant not found");
  });
});
