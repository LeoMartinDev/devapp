# Compact Project Settings — Design Spec

**Date**: 2026-06-29  
**Status**: approved  
**Scope**: frontend only — visual density and layout refinement for the project settings editor and project settings dialog. No backend, schema, or behavior changes.

## Context

The current settings editing experience feels too large and too open. Buttons, inputs, and section spacing dominate the page, and the content stretches too far across the window. The result is functional, but heavier than the rest of Devapp.

The target is a more compact, centered settings surface:

- smaller controls overall;
- a centered content container with stronger side padding;
- cards and grouped areas with a slightly lighter dark surface so they read as distinct regions;
- no behavior change to validation, saving, navigation, or config loading.

## Decisions

1. **Keep the current settings structure.**
   The editor keeps its current sections, process editor, and save flow. This is a visual refinement, not a layout rewrite.

2. **Reduce the perceived scale of controls.**
   Use compact sizing for the settings form controls and secondary actions so the page reads as denser and more deliberate.

3. **Center the content and narrow the reading width.**
   The main editor content should sit inside a centered container with clear horizontal padding, instead of feeling stretched edge-to-edge.

4. **Differentiate cards from the canvas.**
   Process cards and summary blocks should use a slightly raised dark background so they separate from the page without becoming bright panels.

5. **Keep the change local to settings surfaces.**
   Other app areas should retain their current sizing and visual rhythm unless they directly reuse the same settings components.

## Visual Direction

### Overall feel

The settings editor should feel like a compact workspace panel:

- calm rather than spacious;
- readable without filling the whole window;
- dense, but not cramped;
- visually grouped through subtle surface contrast instead of large type and heavy borders.

### Main content

The page content should be centered and bounded by a narrower max width. Side padding should be increased so the editor does not touch the visual edges of the viewport. The result should feel more like a focused inspector than a full-width form.

### Cards and grouped blocks

Process blocks and summary cards should use a slightly lighter surface than the canvas. The contrast should be subtle: enough to separate interactive regions, but not so strong that the page starts to feel boxed-in.

### Controls

Buttons, inputs, and selects inside the settings surfaces should use smaller dimensions where appropriate. The goal is a quieter density, not a dramatic redesign of component styling.

## Component-Level Design

### ConfigEditor

`ConfigEditor.svelte` owns the page-level settings layout. It should:

- use a narrower centered container for the page body;
- add more interior horizontal padding;
- adjust process card surfaces to a raised dark tone;
- keep the current section structure and footer behavior intact.

### ProjectSettingsDialog

`ProjectSettingsDialog.svelte` should feel visually aligned with the compact editor:

- tighter field spacing;
- smaller overall footprint;
- calmer footer actions;
- the same validation and save/remove behavior.

### Shared form controls

`TextField.svelte`, `SelectField.svelte`, and `Button.svelte` may need compact variants or smaller defaults when used in settings surfaces. Any change to shared controls must preserve their current behavior outside settings unless explicitly overridden by props or local class usage.

## Interaction and Behavior

No interaction changes are intended:

- validation remains local and inline;
- save/remove flows stay the same;
- section navigation stays the same;
- dirty-state handling stays the same;
- configuration loading and preview behavior stay the same.

The only difference is visual density and surface treatment.

## Layout Rules

1. Center the settings content within a bounded width.
2. Increase horizontal breathing room around the editor.
3. Reduce the scale of controls used in settings.
4. Use subtle raised surfaces for cards and grouped content.
5. Avoid changing unrelated app areas.

## Files Expected To Change

- `src/lib/components/ConfigEditor.svelte`
- `src/lib/components/ProjectSettingsDialog.svelte`
- `src/lib/components/ui/TextField.svelte`
- `src/lib/components/ui/SelectField.svelte`
- `src/lib/components/ui/Button.svelte`
- `src/lib/components/ui/Dialog.svelte` *(only if needed for centered panel sizing)*

## Validation Plan

After implementation, validate that:

- the settings editor reads as centered and compact;
- form controls no longer feel oversized;
- the cards still separate regions without looking loud;
- the project settings dialog matches the same density;
- the normal frontend check/build still passes.

## Out of Scope

- backend changes;
- schema changes;
- broad theme redesign;
- changing how settings work;
- changes to other app surfaces that do not reuse these settings components.
