import { describe, expect, it, vi } from "vitest";
import { render } from "@testing-library/svelte";

import ProcessForm from "./ProcessForm.svelte";
import type { ProcessForm as ProcessFormState } from "$lib/config/editorModel";

const process: ProcessFormState = {
  id: "process-1",
  name: "api",
  kind: "service",
  cmd: "deno task dev",
  envRows: [],
  dependencies: [],
  readyEnabled: false,
  readyType: "http",
  httpUrl: "http://localhost:3000",
  logPattern: "ready",
  logRegex: false,
  delayDurationMs: 1000,
  commandCmd: "",
  intervalMs: null,
  timeoutMs: 60000,
};

describe("ProcessForm", () => {
  it("uses an icon-only remove action for the selected process", () => {
    const { getByRole, queryByText } = render(ProcessForm, {
      props: {
        process,
        processCount: 2,
        processIssue: () => null,
        onRemove: vi.fn(),
      },
    });

    expect(getByRole("button", { name: "Remove process" })).toBeInTheDocument();
    expect(queryByText("Remove process")).toBeNull();
  });
});