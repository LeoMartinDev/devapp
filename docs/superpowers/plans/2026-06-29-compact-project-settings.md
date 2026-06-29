# Compact Project Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the project settings editing surfaces feel more compact, centered, and visually separated from the canvas without changing behavior.

**Architecture:** Keep the existing settings information architecture. Introduce compact density support in the shared text/select fields, then apply it only to settings surfaces. Narrow the settings content width, increase side padding, and give grouped cards a subtly raised dark surface so the editor feels denser and calmer.

**Tech Stack:** SvelteKit, Tauri, Tailwind CSS v4, Deno

## Global Constraints

- **frontend only — visual density and layout refinement for the project settings editor and project settings dialog. No backend, schema, or behavior changes.**
- **Keep the current settings structure.**
- **Reduce the perceived scale of controls.**
- **Center the content and narrow the reading width.**
- **Differentiate cards from the canvas.**
- **Keep the change local to settings surfaces.**

---

### Task 1: Add compact density to shared form fields

**Files:**
- Modify: `src/lib/components/ui/TextField.svelte`
- Modify: `src/lib/components/ui/SelectField.svelte`

**Interfaces:**
- Consumes: existing `label`, `value`, `error`, and `class` props.
- Produces: optional `density?: "default" | "compact"` prop on both fields, with compact mode using shorter height and slightly tighter horizontal padding.

- [ ] **Step 1: Update the shared field APIs**
  - Add `density?: "default" | "compact"` to both components.
  - Keep the current defaults unchanged for existing callers.

- [ ] **Step 2: Wire compact sizing into the class logic**
  - `TextField` should switch from `h-9 px-3` to a compact `h-8 px-2.5` style when `density === "compact"`.
  - `SelectField` should do the same.

- [ ] **Step 3: Run a focused frontend check**
  - Run: `deno task check`
  - Expected: the typecheck/lint pass should still succeed, or any failures should point at the new prop usage only.

---

### Task 2: Tighten the main settings page layout

**Files:**
- Modify: `src/lib/components/ConfigEditor.svelte`

**Interfaces:**
- Consumes: the new `density="compact"` prop from `TextField` and `SelectField`.
- Produces: a narrower, centered page body with subtler card surfaces and compact control usage.

- [ ] **Step 1: Narrow and center the page content**
  - Replace the current wide content wrapper (`max-w-245`) with a narrower centered container.
  - Increase horizontal padding so the editor sits further from the viewport edges.

- [ ] **Step 2: Make grouped blocks read as lighter dark cards**
  - Change the summary cards and process cards away from `bg-surface/10` / `bg-surface/40` toward `bg-surface-raised`-based backgrounds.
  - Keep borders subtle; the cards should separate regions without becoming bright panels.

- [ ] **Step 3: Apply compact controls inside the editor**
  - Pass `density="compact"` to all settings-page `TextField` and `SelectField` instances.
  - Keep the existing button hierarchy, but use the smaller `size="sm"` buttons where they are not already compact.

- [ ] **Step 4: Verify the page still behaves the same**
  - Open the settings page manually and confirm section navigation, process editing, and preview toggling still work.

---

### Task 3: Compact the project settings dialog and its fields

**Files:**
- Modify: `src/lib/components/ProjectSettingsDialog.svelte`
- Modify: `src/lib/components/ConfigEditor.svelte` *(footer button sizing if needed after the layout change)*

**Interfaces:**
- Consumes: the compact field props from Task 1.
- Produces: a smaller, centered project settings dialog that matches the denser visual language.

- [ ] **Step 1: Apply compact field density in the dialog**
  - Pass `density="compact"` to the Name, Base directory, and Configuration source fields.

- [ ] **Step 2: Tighten the dialog presentation**
  - Reduce unnecessary vertical breathing room in the dialog body.
  - Keep the remove confirmation flow and validation unchanged.

- [ ] **Step 3: Make the action buttons feel less oversized**
  - Use `size="sm"` for the dialog footer buttons if they still feel too large after the field changes.

- [ ] **Step 4: Verify the dialog still saves and removes correctly**
  - Open the dialog, save an edit, and confirm the remove confirmation still works.

---

### Task 4: Run the repository validation commands

**Files:**
- None

**Interfaces:**
- Consumes: the updated frontend files from Tasks 1-3.
- Produces: confirmation that the compact layout changes did not break the project checks.

- [ ] **Step 1: Run the repository check**
  - Run: `deno task check`
  - Expected: pass.

- [ ] **Step 2: Run the repository build**
  - Run: `deno task build`
  - Expected: pass.

- [ ] **Step 3: Commit the finished work**
  - Commit the implementation with a message consistent with the repo’s existing style.
