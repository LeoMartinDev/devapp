# UI/UX Polish — 10-Point Design

**Date**: 2026-06-26
**Status**: approved

## Overview

Design for 10 UI/UX improvements balanced across visual quality, robustness, and
ergonomics on the Devapp desktop app (SvelteKit + Tauri, Tailwind CSS v4, dark
theme). The work is scoped to the existing frontend; the Rust backend is
unchanged.

---

## 1. Global Spinner (not skeletons)

**Decision**: No skeleton loaders. A single spinner that replaces the Run/Stop
button icon when the runtime is busy.

**Requirements**:
- Same dimensions as the Run/Stop button (no layout shift).
- Icon-only (no text).
- Spinning animation (rotate).
- Replaces the current disabled/greyed button behavior.

**Implementation**: Add a `busy` variant to `RunStopButton.svelte` that renders
an animated spinner SVG in place of the play/stop icon.

---

## 2. Status Dot & Dialog Transitions

**Kept**:
- Process status dot: morph/glow animation on state change (e.g. running →
  ready). Uses `prefers-reduced-motion` guard.
- Dialog open/close: backdrop fade 150ms + dialog scale(0.95→1) 150ms ease-out.

**Rejected**:
- No crossfade/slide when switching between sidebar views (log ↔ terminal).
- No button press micro-interaction (keep sharp).

---

## 3. Toast / Notification System

**Decision**: Lightweight toast system, positioned bottom-right, stacking
vertically. No notification center needed for V1.

**Toast types**:
- Success (green check, e.g. "Config saved")
- Info (blue, e.g. "Logs copied — 142 lines")
- Error (red, e.g. "Failed to start api — port 3000 in use")

**Behavior**:
- Auto-dismiss after 4s for success/info.
- Error toasts persist until dismissed or action taken (Retry).
- Max 3 toasts visible at once (oldest removed).
- Slide-in from right edge.

**Component**: New `Toast.svelte` rendered in a portal at the bottom of the app
shell (z-index above dialogs).

---

## 4. LogViewer — Stream Differentiation

**Kept**:
- Colored left-border bar on each log line (3px): stdout = muted white, stderr =
  red, system = blue.
- "● Process ready" icon prefix for system-ready messages.

**Rejected**:
- No line grouping or timestamp collapsing.
- No toolbar redesign (keep current icons).
- No relative timestamps.

**Implementation**: Modify `LogViewer.svelte` to add a colored border-left per
stream type, and prefix system "ready" lines with a dot icon.

---

## 5. Keyboard Accessibility

**Sidebar actions on focus**: Add `:focus-visible` and `:focus-within` rules so
the stop/start/restart buttons in `ProcessList.svelte` become visible when a row
receives keyboard focus (not only on hover).

**Focus ring**: Use `*:focus-visible { outline: 2px solid var(--color-blue-400);
outline-offset: 2px }` globally. Suppress `:focus` outlines.

**LogViewer toolbar as a single tab stop**: Wrap the toolbar action buttons in a
`role="toolbar"` container with `aria-label`. Tab enters/exits the group; arrow
keys navigate internally. This reduces the current 4 Tab stops to 1.

---

## 6. Dirty State Protection (ConfigEditor)

**Unsaved changes dialog**: When the ConfigEditor has `isDirty === true` and the
user triggers close (Escape, overlay click, Cancel), show a confirmation dialog:
"Unsaved changes — Discard them?" with "Keep editing" (default / Escape) and
"Discard" (danger) buttons.

**Modified field indicator**: Add a 2px left border accent (`#fbbf24` yellow) to
form fields whose value differs from the loaded config. Applies to text inputs,
selects, and env var rows.

**Footer badge**: Show a "Modified" badge (yellow) in the ConfigEditor footer
when `isDirty` is true.

---

## 7. Onboarding & Empty States

**Welcome screen** (no project loaded):
- Centered card with app icon, "Welcome to Devapp" title, one-line description,
  and a prominent "+ Register project" CTA button.
- Below: "RECENT PROJECTS" section with clickable project rows (name + path).
- Component: new `WelcomeScreen.svelte`, shown in the main content area when
  `projectId === null` and no session is active.

**Terminal empty state** (refinement):
- Keep current icon + text + CTA.
- Add "or press Ctrl+T" hint next to the button.

**Rejected**: No multi-step wizard or guided tour for V1.

---

## 8. Typography & Spacing Consistency

**Form fields**: Enforce `h-9` (36px) height for all text inputs and selects.
Replace any `h-10` or inconsistent padding values in `TextField.svelte`,
`SelectField.svelte`, and `ProjectSettingsDialog.svelte`.

**Sidebar visual hierarchy**:
- Split into two labeled sections: "PROCESSES" and "TERMINALS", separated by a
  subtle divider line.
- Each process/terminal row gets a 6px colored status dot (replacing current
  icon/text status) and a compact badge for the status label.
- Show process count ("3/4 done") next to the section header when a session is
  active.

**Density**: Compact layout by default (current behavior, refined). A
comfortable mode toggle is deferred to a future iteration.

---

## 9. System Preferences

**`prefers-reduced-motion`**: Add a global CSS rule:

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

This disables the status dot pulse, dialog transitions, and any future
animations when the user's OS setting requests reduced motion.

**Focus visibility**: As described in point 5 — `:focus-visible` only, never
`:focus`. The existing `@theme` block in `app.css` gains the rule.

**`prefers-color-scheme`**: Already correct (forced dark theme). No action
needed.

---

## Implementation Order

By dependency and risk:

1. **System preferences** — CSS-only, no component changes, low risk.
2. **Global spinner** — isolated to `RunStopButton.svelte`.
3. **Keyboard accessibility** — CSS + small component refactors.
4. **Dirty state protection** — adds dialog + field indicators to ConfigEditor.
5. **Onboarding & empty states** — new `WelcomeScreen` component.
6. **Typography & spacing** — CSS adjustments across multiple components.
7. **Toast system** — new component + store integration.
8. **Status dot & dialog transitions** — depends on system preferences being in
   place.
9. **LogViewer stream differentiation** — CSS change to log lines.
