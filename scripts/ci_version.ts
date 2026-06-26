import { computeVersion } from "./version_calc.ts";

const FILES = {
  tauriConf: "src-tauri/tauri.conf.json",
  packageJson: "package.json",
  cargoToml: "src-tauri/Cargo.toml",
};

const dryRun = Deno.args.includes("--dry-run");

async function git(args: string[]): Promise<string> {
  const cmd = new Deno.Command("git", {
    args,
    stdout: "piped",
    stderr: "piped",
  });
  const { code, stdout, stderr } = await cmd.output();
  const out = new TextDecoder().decode(stdout);
  const err = new TextDecoder().decode(stderr);
  if (code !== 0) {
    throw new Error(`git ${args.join(" ")} failed (${code}): ${err || out}`);
  }
  return out.trim();
}

async function patchJson(path: string, version: string): Promise<void> {
  const text = await Deno.readTextFile(path);
  const json = JSON.parse(text);
  json.version = version;
  await Deno.writeTextFile(path, JSON.stringify(json, null, 2) + "\n");
}

async function patchCargoVersion(path: string, version: string): Promise<void> {
  let text = await Deno.readTextFile(path);
  // Only the [package] version line starts the line with `version =`.
  // Dependency versions live inside `{ ... }` on lines that start with the dep name.
  text = text.replace(
    /^version\s*=\s*"[^"]*"/m,
    `version = "${version}"`,
  );
  await Deno.writeTextFile(path, text);
}

async function main(): Promise<void> {
  await git(["fetch", "--tags", "--force"]);
  const tagList = await git(["tag", "-l", "v*"]);
  const existingTags = tagList ? tagList.split("\n").filter(Boolean) : [];

  const { version, tag } = computeVersion(new Date(), existingTags);

  if (dryRun) {
    console.log(`VERSION=${version}`);
    console.log(`TAG=${tag}`);
    console.log(`(dry-run: no files patched, no GITHUB_ENV written)`);
    return;
  }

  await patchJson(FILES.tauriConf, version);
  await patchJson(FILES.packageJson, version);
  await patchCargoVersion(FILES.cargoToml, version);

  const ghEnv = Deno.env.get("GITHUB_ENV");
  if (ghEnv) {
    const line = `VERSION=${version}\nTAG=${tag}\n`;
    await Deno.writeTextFile(ghEnv, line, { append: true });
  }

  console.log(`VERSION=${version}`);
  console.log(`TAG=${tag}`);
}

main();
