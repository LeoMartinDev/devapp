import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";

import WindowControls from "./WindowControls.svelte";

describe("WindowControls", () => {
  it("does not render native controls in plain web mode", () => {
    const { queryByRole } = render(WindowControls);

    expect(queryByRole("button", { name: "Minimize" })).toBeNull();
    expect(queryByRole("button", { name: "Maximize" })).toBeNull();
    expect(queryByRole("button", { name: "Restore" })).toBeNull();
    expect(queryByRole("button", { name: "Close" })).toBeNull();
  });
});
