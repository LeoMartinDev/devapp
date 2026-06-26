export interface VersionResult {
  version: string;
  tag: string;
}

/**
 * Compute a CalVer version of the form YY.M.D-n, where:
 *  - YY = year - 2000 (Windows MSI requires major <= 255)
 *  - M, D have no leading zeros (strict SemVer forbids leading zeros)
 *  - n is (max existing counter for this date) + 1, defaulting to 1
 */
export function computeVersion(now: Date, existingTags: string[]): VersionResult {
  const yy = now.getFullYear() - 2000;
  const m = now.getMonth() + 1;
  const d = now.getDate();
  const prefix = `${yy}.${m}.${d}`;
  const tagPrefix = `v${prefix}-`;

  let max = 0;
  for (const tag of existingTags) {
    if (!tag.startsWith(tagPrefix)) continue;
    const suffix = tag.slice(tagPrefix.length);
    const n = Number.parseInt(suffix, 10);
    if (String(n) === suffix && n > max) max = n;
  }

  const n = max + 1;
  const version = `${prefix}-${n}`;
  return { version, tag: `v${version}` };
}
