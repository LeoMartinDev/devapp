import { describe, expect, it } from "vitest";

import { setWindowTitle } from "./client";

describe("setWindowTitle", () => {
  it("is a no-op in plain web mode", async () => {
    await expect(setWindowTitle("devapp")).resolves.toBeUndefined();
  });
});
