export interface ShortcutEntry {
  key: string;
  description: string;
  handler: () => void;
  guard?: () => boolean;
}

export interface ShortcutRegistryOptions {
  typingGuard?: boolean;
}

function eventToCombo(event: KeyboardEvent): string {
  const parts: string[] = [];
  if (event.ctrlKey || event.metaKey) parts.push("Mod");
  if (event.shiftKey) parts.push("Shift");
  if (event.altKey) parts.push("Alt");
  parts.push(event.key);
  return parts.join("+");
}

function isTypingTarget(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  return (
    el instanceof HTMLInputElement ||
    el instanceof HTMLTextAreaElement ||
    el instanceof HTMLSelectElement ||
    el?.isContentEditable === true
  );
}

export function createShortcutRegistry(
  shortcuts: ShortcutEntry[],
  opts: ShortcutRegistryOptions = {},
): (event: KeyboardEvent) => void {
  const map = new Map<string, ShortcutEntry>();
  for (const entry of shortcuts) {
    if (map.has(entry.key)) {
      console.warn(
        `[ShortcutRegistry] duplicate binding "${entry.key}" — later entry overrides earlier`,
      );
    }
    map.set(entry.key, entry);
  }

  const typingGuard = opts.typingGuard ?? true;

  return (event: KeyboardEvent) => {
    if (typingGuard && isTypingTarget(event.target)) return;

    const combo = eventToCombo(event);
    const entry = map.get(combo);
    if (!entry) return;
    if (entry.guard && !entry.guard()) return;

    event.preventDefault();
    entry.handler();
  };
}
