# Settings A1 Minimal Redesign — Design Spec

**Date**: 2026-06-27
**Status**: approved
**Scope**: frontend only — settings UI polish for the full-page ConfigEditor. No backend or schema changes.

## Context

The current settings UI in Devapp feels heavier than the product needs. The main
issues raised during brainstorming were:

- buttons feel too large and too prominent;
- the Save button label is noisy because it embeds dirty state with `*`;
- contrast is inconsistent across the page;
- too many nested cards make the form feel amateur and widget-heavy;
- the UI needs a more mature desktop-tool feel, closer to Cursor Agent, while
  staying dense and operational.

The full-page settings editor is already the correct structural direction for
Devapp. The redesign should keep that information architecture, but simplify the
visual language so the page reads like a compact inspector instead of a stack of
interactive panels.

## Decisions

1. **Adopt the A1 visual direction.**
   The settings page uses a continuous inspector layout inspired by Cursor's
   desktop settings: calm left navigation, one main configuration column, and as
   few containers as possible.

2. **Make process configuration vertical.**
   Process editing is no longer presented as a composition of sub-cards. The
   right column becomes a single vertical configuration flow: identity,
   environment variables, dependencies, readiness.

3. **Reduce action weight aggressively.**
   Add actions remain compact secondary buttons. Remove actions become icon-only
   controls using a neutral trash icon with no visible outline at rest.

4. **Remove dirty-state noise from the Save button.**
   The primary CTA remains simply `Save`. Dirty state is communicated elsewhere,
   not through `Save *` or other label mutations.

5. **Use icons sparingly.**
   Icons are allowed for navigation and icon-only destructive actions. They are
   not used decoratively on every section or button.

6. **Keep behavior stable.**
   This redesign changes presentation, density, and emphasis. It does not change
   save semantics, touched-field validation rules, section navigation behavior,
   or backend configuration logic.

## Visual Direction

### Overall feel

The target feel is a restrained desktop settings surface:

- quiet by default;
- dense, but not cramped;
- structured by spacing, typography, and dividers instead of repeated cards;
- interactive without looking playful;
- visually close to a tooling UI, not a marketing UI.

The page should feel more like a document or inspector than a dashboard.

### Left rail

The left rail remains split into two clear groups:

- settings sections;
- processes.

Each row is mostly typographic navigation. Active rows use a soft background and
slightly stronger contrast, but do not become mini-cards. Badges and heavy
status chrome are avoided.

### Main content column

The main content column keeps a controlled readable width. It should not become
fully liquid across the entire window. The content reads top-to-bottom with a
steady vertical rhythm.

Global sections such as `General`, `Environment`, `Processes`, and `YAML
preview` share the same visual logic:

- short title;
- optional short support text;
- content area;
- divider before the next subsection when useful.

### Process configuration

The selected process is edited in a single vertical flow. The intended order is:

1. identity;
2. environment variables;
3. dependencies;
4. readiness.

Each subsection is introduced by a small title and separated with spacing or a
thin divider. Sub-cards are intentionally removed. The user should feel like
they are scanning one coherent sheet, not navigating between nested panels.

## Component-Level Design

### ConfigEditor

`ConfigEditor.svelte` remains the owning layout surface. The redesign changes:

- page spacing;
- perceived hierarchy between the left rail and the main column;
- section wrappers and dividers;
- footer status treatment;
- button emphasis.

The section navigation model and process-selection model stay unchanged.

### ProcessForm

`ProcessForm.svelte` should stop presenting the process header as a strong
top-level action row. The process identity fields remain aligned in a simple
grid, but the surrounding chrome becomes lighter.

Expected changes:

- lighter top separation;
- no visually loud remove button;
- compact field heights;
- fields arranged to support vertical reading, not card grouping.

### EnvEditor

`EnvEditor.svelte` should keep the current data model and row controls, but the
visual treatment becomes quieter.

Expected changes:

- smaller secondary `Add variable` button;
- remove action replaced with an icon-only neutral trash button;
- empty state simplified so it reads as a quiet inline absence, not a large callout;
- row spacing tuned for dense repeated editing.

### DependencyEditor

`DependencyEditor.svelte` follows the same treatment as environment rows:

- compact add button;
- icon-only neutral remove action;
- lighter empty state;
- repeated rows optimized for scanability.

### ReadyCheckEditor

`ReadyCheckEditor.svelte` stays functionally identical but should visually read
as another part of the same continuous process sheet. The current boxed section
should be flattened so it no longer looks like a nested card inside a card.

### Button system

`Button.svelte` should support this redesign through emphasis and size changes:

- smaller default secondary controls where appropriate;
- primary button still clearly primary, but not oversized;
- icon-only affordance for remove actions, likely as either a new variant,
  a new size, or a dedicated pattern consumed by settings components.

The key requirement is that remove actions do not look like warning CTAs at rest.

## Interaction States

### Save and dirty state

The Save button label remains constant: `Save`.

Dirty state moves out of the CTA and into a quieter status surface:

- footer status text is the primary location;
- a subtle supplementary header badge is allowed, but optional;
- `Save *` is removed.

This keeps the primary action calm while preserving state awareness.

### Remove actions

Remove actions use icon-only trash affordances with these rules:

- no visible outline at rest;
- no red styling at rest;
- neutral text/subtle color by default;
- slightly stronger contrast or faint hover fill on hover/focus;
- red reserved for real destructive confirmation or error surfaces, not the
  resting state of every remove action.

### Validation

Validation behavior stays the same logically, but its visual expression becomes
more restrained:

- invalid touched fields still show inline feedback;
- the field itself gets a clear but not heavy invalid state;
- messages stay close to the offending field;
- global error summaries remain secondary to local correction.

The page should not visually collapse into an alert screen during normal editing.

### Hover and focus

Hover and focus treatments should stay light and tooling-like:

- subtle background change;
- fine border or outline reinforcement;
- no loud animation;
- clear keyboard focus for icon-only remove controls.

## Layout Rules

1. Keep the left rail compact and predominantly textual.
2. Keep the main editor column width controlled for readability.
3. Preserve a simple grid for fields, but organize the whole process view as a
   vertical document.
4. Prefer separators and rhythm over additional containers.
5. Avoid introducing a new settings search feature in this iteration, even if
   Cursor is a visual inspiration.

## Files Expected To Change

- `src/lib/components/ConfigEditor.svelte`
- `src/lib/components/ProcessForm.svelte`
- `src/lib/components/EnvEditor.svelte`
- `src/lib/components/DependencyEditor.svelte`
- `src/lib/components/ReadyCheckEditor.svelte`
- `src/lib/components/ui/Button.svelte`
- `src/app.css`

Additional small UI helper changes are acceptable if they directly support the
above design. Backend files, validation logic, runtime logic, and YAML schema
files are out of scope.

## Validation Plan

Manual validation should confirm the redesign still feels clear when editing a
real multi-process configuration.

Focus areas:

- a long process form remains scannable without sub-cards;
- add buttons no longer dominate the page;
- icon-only remove actions remain discoverable on hover and keyboard focus;
- dirty state is obvious enough without mutating the Save button label;
- inline validation remains legible and local;
- the left rail still clearly separates sections from processes;
- the YAML preview still feels integrated into the same settings language.

Executable validation after implementation should include the normal frontend
check used by the repo, plus manual inspection of the settings page.

## Out Of Scope

- backend changes;
- YAML schema changes;
- new settings search functionality;
- changes to save semantics;
- changes to touched-field validation rules;
- changes to process editing capabilities;
- broader application theming beyond what the settings redesign requires.
