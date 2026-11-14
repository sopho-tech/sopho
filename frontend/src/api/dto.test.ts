import { describe, expect, test } from "vitest";
import { ApiError } from "src/api/dto";

const REQUEST_ID = "3f1c2a9e-5b7d-4c1a-9e2f-0a1b2c3d4e5f";

describe("API error", () => {
  test("shows the request id with a server error", () => {
    const serverError = new ApiError(500, {
      error: "Database error",
      request_id: REQUEST_ID,
    });

    expect(serverError.message).toBe(
      `Database error (Request ID: ${REQUEST_ID})`,
    );
    expect(serverError.requestId).toBe(REQUEST_ID);
  });

  test("keeps the backend's message as it is for a refusal", () => {
    const refusal = new ApiError(403, {
      message: "This tenant is suspended. Contact support",
      error_code: "TENANT_SUSPENDED",
      request_id: REQUEST_ID,
    });

    expect(refusal.message).toBe("This tenant is suspended. Contact support");
    expect(refusal.requestId).toBe(REQUEST_ID);
  });

  test("shows a server error without a request id as it is", () => {
    const serverError = new ApiError(502, { error: "Bad gateway" });

    expect(serverError.message).toBe("Bad gateway");
    expect(serverError.requestId).toBeUndefined();
  });
});
