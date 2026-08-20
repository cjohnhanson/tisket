#!/usr/bin/env node
// The binary lives in a per-platform package under optionalDependencies,
// so a package manager installs only the one that matches. This resolves
// it through node's own lookup rather than a hardcoded node_modules path,
// and execs it.
//
// Nothing is downloaded at install time. There is no postinstall, so an
// offline install, an air-gapped runner, and `--ignore-scripts` all work.
const { platform, arch, env } = process;
const { spawnSync, execSync } = require("child_process");

// `libc` in a platform package is a hint, and package managers disagree
// about honouring it, so musl is detected here too.
function isMusl() {
  let stderr;
  try {
    stderr = execSync("ldd --version", { stdio: ["pipe", "pipe", "pipe"] });
  } catch (err) {
    stderr = err.stderr;
  }
  return String(stderr).indexOf("musl") > -1;
}

const PLATFORMS = {
  darwin: {
    arm64: "@tisket/cli-darwin-arm64/tisket",
    x64: "@tisket/cli-darwin-x64/tisket",
  },
  "linux-musl": {
    arm64: "@tisket/cli-linux-arm64-musl/tisket",
    x64: "@tisket/cli-linux-x64-musl/tisket",
  },
};

const key = platform === "linux" && isMusl() ? "linux-musl" : platform;
const rel = env.TISKET_BINARY ? null : PLATFORMS?.[key]?.[arch];
const bin = env.TISKET_BINARY || (rel && require.resolve(rel));

if (!bin) {
  console.error(
    `tisket ships no prebuilt binary for ${platform} ${arch}. ` +
      "Install it with `cargo install tisket`, or set TISKET_BINARY to a path."
  );
  process.exitCode = 1;
} else {
  const result = spawnSync(bin, process.argv.slice(2), {
    shell: false,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  process.exitCode = result.status;
}
