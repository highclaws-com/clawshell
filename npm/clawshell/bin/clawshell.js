#!/usr/bin/env node

const { execFileSync } = require("child_process");
const path = require("path");

const PLATFORMS = {
  "darwin-arm64": "@runta-dev/clawshell-darwin-arm64",
  "linux-arm64": "@runta-dev/clawshell-linux-arm64",
  "linux-x64": "@runta-dev/clawshell-linux-x64",
};

const platform = `${process.platform}-${process.arch}`;
const pkg = PLATFORMS[platform];

if (!pkg) {
  console.error(
    `clawshell: unsupported platform ${platform}. Supported: ${Object.keys(PLATFORMS).join(", ")}`
  );
  process.exit(1);
}

let binPath;
try {
  const pkgDir = path.dirname(require.resolve(`${pkg}/package.json`));
  binPath = path.join(pkgDir, "bin", "clawshell");
} catch {
  console.error(
    `clawshell: could not find package ${pkg}.\n` +
      `Make sure optional dependencies are installed (do not use --no-optional).`
  );
  process.exit(1);
}

const result = require("child_process").spawnSync(binPath, process.argv.slice(2), {
  stdio: "inherit",
});

if (result.error) {
  console.error(`clawshell: failed to start: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
