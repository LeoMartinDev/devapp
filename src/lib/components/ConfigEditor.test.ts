import { afterEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, waitFor } from "@testing-library/svelte";

import ConfigEditor from "./ConfigEditor.svelte";
import { runtimeStore } from "$lib/stores/runtime.svelte";
import type { ProjectConfigDocument, ProjectRecord } from "$lib/types";

const loadedProject: ProjectRecord = {
  id: "project-1",
  name: "Demo app",
  baseDir: "/tmp/demo-app",
  configSource: "projectFile",
  configPath: "/tmp/demo-app/devapp.yml",
  createdAt: "2026-06-27T00:00:00Z",
  updatedAt: "2026-06-27T00:00:00Z",
};

const loadedDocument: ProjectConfigDocument = {
  project: loadedProject,
  yaml: "version: 1\nprocesses:\n  api:\n    kind: service\n    cmd: deno task dev\n  worker:\n    kind: task\n    cmd: deno task worker\n",
  config: {
    version: 1,
    env: {
      GLOBAL_TOKEN: "demo",
    },
    processes: {
      api: {
        kind: "service",
        cmd: "deno task dev",
        env: {
          API_TOKEN: "demo",
        },
        dependsOn: {},
        ready: {
          type: "log",
          pattern: "ready",
          regex: false,
          timeoutMs: 60000,
        },
      },
      worker: {
        kind: "task",
        cmd: "deno task worker",
        env: {},
        dependsOn: {
          api: "ready",
        },
      },
    },
  },
};

afterEach(() => {
  vi.restoreAllMocks();
});

function renderLoadedEditor() {
  vi.spyOn(runtimeStore, "loadConfig").mockResolvedValue(loadedDocument);

  return render(ConfigEditor, {
    props: {
      open: true,
      project: loadedProject,
      onClose: vi.fn(),
      mode: "page",
    },
  });
}

describe("ConfigEditor", () => {
  it("renders as a full page with a go back control in page mode", async () => {
    const onClose = vi.fn();
    const { getByRole, queryByRole } = render(ConfigEditor, {
      props: {
        open: true,
        project: null,
        onClose,
        mode: "page",
      },
    });

    expect(queryByRole("dialog")).toBeNull();

    const backButton = getByRole("button", { name: "Go back" });
    expect(getByRole("heading", { name: "Runtime configuration" })).toBeInTheDocument();

    await fireEvent.click(backButton);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("shows a dedicated settings navigation in page mode", () => {
    const { getByRole } = render(ConfigEditor, {
      props: {
        open: true,
        project: null,
        onClose: vi.fn(),
        mode: "page",
      },
    });

    const navigation = getByRole("navigation", { name: "Settings sections" });

    expect(navigation).toBeInTheDocument();
    expect(getByRole("link", { name: "General" })).toBeInTheDocument();
    expect(getByRole("link", { name: "Environment" })).toBeInTheDocument();
    expect(getByRole("link", { name: "YAML preview" })).toBeInTheDocument();
  });

  it("tracks the active settings section and scrolls to it from the left navigation", async () => {
    const { getByRole } = render(ConfigEditor, {
      props: {
        open: true,
        project: null,
        onClose: vi.fn(),
        mode: "page",
      },
    });

    const generalLink = getByRole("link", { name: "General" });
    const previewLink = getByRole("link", { name: "YAML preview" });
    const previewSection = getByRole("heading", { name: "YAML preview" }).closest("section") as HTMLElement;
    const scrollIntoView = vi.fn();
    previewSection.scrollIntoView = scrollIntoView;

    expect(generalLink.getAttribute("aria-current")).toBe("page");

    await fireEvent.click(previewLink);

    expect(previewLink.getAttribute("aria-current")).toBe("page");
    expect(generalLink.getAttribute("aria-current")).toBeNull();
    expect(scrollIntoView).toHaveBeenCalledWith({
      behavior: "smooth",
      block: "start",
      inline: "nearest",
    });
  });

  it("keeps the primary action label stable in page mode", () => {
    const { getByRole, queryByRole } = render(ConfigEditor, {
      props: {
        open: true,
        project: null,
        onClose: vi.fn(),
        mode: "page",
      },
    });

    expect(getByRole("button", { name: "Save" })).toBeInTheDocument();
    expect(queryByRole("button", { name: "Save settings" })).toBeNull();
  });

  it("renders the process rail as an accessible listbox and switches the selected process panel after load", async () => {
    const { findByRole, getByRole } = renderLoadedEditor();

    expect(await findByRole("listbox", { name: "Processes" })).toBeInTheDocument();

    const apiOption = getByRole("option", { name: /api/i });
    const workerOption = getByRole("option", { name: /worker/i });

    expect(apiOption).toHaveAttribute("aria-selected", "true");
    expect(getByRole("region", { name: /api/i })).toBeInTheDocument();

    await fireEvent.click(workerOption);

    await waitFor(() => {
      expect(workerOption).toHaveAttribute("aria-selected", "true");
    });

    expect(apiOption).toHaveAttribute("aria-selected", "false");
    expect(getByRole("region", { name: /worker/i })).toBeInTheDocument();
    expect(getByRole("textbox", { name: "Name" })).toHaveValue("worker");
  });

  it("supports keyboard navigation across loaded process options", async () => {
    const { findByRole, getByRole } = renderLoadedEditor();

    await findByRole("region", { name: /api/i });

    const apiOption = getByRole("option", { name: /api/i });
    const workerOption = getByRole("option", { name: /worker/i });

    apiOption.focus();
    expect(apiOption).toHaveFocus();

    await fireEvent.keyDown(apiOption, { key: "ArrowDown" });

    await waitFor(() => {
      expect(workerOption).toHaveAttribute("aria-selected", "true");
    });

    expect(workerOption).toHaveFocus();
    expect(getByRole("region", { name: /worker/i })).toBeInTheDocument();

    await fireEvent.keyDown(workerOption, { key: "Home" });

    await waitFor(() => {
      expect(apiOption).toHaveAttribute("aria-selected", "true");
    });

    expect(apiOption).toHaveFocus();
    expect(getByRole("region", { name: /api/i })).toBeInTheDocument();
  });

  it("shows dirty footer status for a loaded project while keeping the save label stable", async () => {
    const { findByRole, getByRole, getByText } = renderLoadedEditor();

    await findByRole("region", { name: /api/i });

    await fireEvent.input(getByRole("textbox", { name: "Name" }), {
      target: { value: "api-renamed" },
    });

    await waitFor(() => {
      expect(getByText("Unsaved changes in project YAML")).toBeInTheDocument();
    });

    expect(getByRole("button", { name: "Save" })).toBeInTheDocument();
  });
});