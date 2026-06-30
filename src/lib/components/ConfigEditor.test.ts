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
    },
  });
}

describe("ConfigEditor", () => {
  it("renders as a full page settings surface by default", () => {
    const { queryByRole, getByRole } = render(ConfigEditor, {
      props: {
        open: true,
        project: null,
        onClose: vi.fn(),
      },
    });

    expect(queryByRole("dialog")).toBeNull();
    expect(getByRole("button", { name: "Go back" })).toBeInTheDocument();
  });

  it("renders as a full page with a go back control", async () => {
    const onClose = vi.fn();
    const { getByRole, queryByRole } = render(ConfigEditor, {
      props: {
        open: true,
        project: null,
        onClose,
      },
    });

    expect(queryByRole("dialog")).toBeNull();

    const backButton = getByRole("button", { name: "Go back" });

    await fireEvent.click(backButton);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("shows a dedicated settings navigation", () => {
    const { getByRole } = render(ConfigEditor, {
      props: {
        open: true,
        project: null,
        onClose: vi.fn(),
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

  it("keeps the primary action label stable", () => {
    const { getByRole, queryByRole } = render(ConfigEditor, {
      props: {
        open: true,
        project: null,
        onClose: vi.fn(),
      },
    });

    expect(getByRole("button", { name: "Save" })).toBeInTheDocument();
    expect(queryByRole("button", { name: "Save settings" })).toBeNull();
  });

  it("renders one process card per loaded process", async () => {
    const { findByRole, getAllByRole, getByRole } = renderLoadedEditor();

    expect(await findByRole("heading", { name: "Processes" })).toBeInTheDocument();
    expect(getByRole("heading", { name: "Process: api" })).toBeInTheDocument();
    expect(getByRole("heading", { name: "Process: worker" })).toBeInTheDocument();

    const processNameInputs = getAllByRole("textbox", { name: "Name" }) as HTMLInputElement[];
    expect(processNameInputs.map((input) => input.value)).toEqual(expect.arrayContaining(["api", "worker"]));
  });

  it("shows process cards with nested dependency and readiness sections", async () => {
    const { findByRole, getAllByRole } = renderLoadedEditor();

    await findByRole("heading", { name: "Processes" });
    expect(getAllByRole("heading", { name: "Dependencies" })).toHaveLength(2);
    expect(getAllByRole("heading", { name: "Readiness" })).toHaveLength(2);
  });

  it("shows dirty footer status for a loaded project while keeping the save label stable", async () => {
    const { findByRole, getAllByRole, getByRole, getByText } = renderLoadedEditor();

    await findByRole("heading", { name: "Process: api" });

    await fireEvent.input(getAllByRole("textbox", { name: "Name" })[0], {
      target: { value: "api-renamed" },
    });

    await waitFor(() => {
      expect(getByText("Unsaved changes in project YAML")).toBeInTheDocument();
    });

    expect(getByRole("button", { name: "Save" })).toBeInTheDocument();
  });
});