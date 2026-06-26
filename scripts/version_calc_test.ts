import { computeVersion } from "./version_calc.ts";

function assertEqual<T>(actual: T, expected: T, msg?: string) {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  if (a !== e) {
    throw new Error(
      `Assertion failed${msg ? `: ${msg}` : ""}\n  expected: ${e}\n  actual:   ${a}`,
    );
  }
}

Deno.test("first build of the day gets counter 1", () => {
  const now = new Date(2026, 5, 26); // 2026-06-26 (month is 0-indexed)
  assertEqual(computeVersion(now, []), {
    version: "26.6.26-1",
    tag: "v26.6.26-1",
  });
});

Deno.test("second build same day increments counter", () => {
  const now = new Date(2026, 5, 26);
  assertEqual(computeVersion(now, ["v26.6.26-1"]), {
    version: "26.6.26-2",
    tag: "v26.6.26-2",
  });
});

Deno.test("ignores tags from other days", () => {
  const now = new Date(2026, 5, 26);
  assertEqual(
    computeVersion(now, ["v26.6.25-3", "v26.7.1-1", "v25.12.31-9"]),
    { version: "26.6.26-1", tag: "v26.6.26-1" },
  );
});

Deno.test("ignores non-calver tags", () => {
  const now = new Date(2026, 5, 26);
  assertEqual(
    computeVersion(now, ["v0.1.0", "latest", "26.6.26-1-bogus"]),
    { version: "26.6.26-1", tag: "v26.6.26-1" },
  );
});

Deno.test("uses max counter + 1 when there are gaps", () => {
  const now = new Date(2026, 5, 26);
  assertEqual(computeVersion(now, ["v26.6.26-1", "v26.6.26-5"]), {
    version: "26.6.26-6",
    tag: "v26.6.26-6",
  });
});

Deno.test("no leading zeros for single-digit month and day", () => {
  const now = new Date(2026, 1, 9); // 2026-02-09
  assertEqual(computeVersion(now, []), {
    version: "26.2.9-1",
    tag: "v26.2.9-1",
  });
});
