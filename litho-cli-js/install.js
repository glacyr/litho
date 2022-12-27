const fs = require("fs");
const https = require("https");
const os = require("os");
const path = require("path");
const zlib = require("zlib");

function architecture() {
  // TODO: this doesn't work for some architectures (e.g. ppc64 doesn't match
  // the powerpc64 that Rust uses), but we don't support those architectures
  // anyway.
  switch (os.arch()) {
    case "x64":
      switch (os.platform()) {
        case "darwin":
          if (os.cpus()[0].model.includes("Apple")) {
            return "aarch64";
          }
        default:
          return "x86_64";
      }
    case "arm64":
      return "aarch64";
    default:
      return os.arch();
  }
}

function vendor() {
  switch (os.platform()) {
    case "darwin":
      return "apple";
    case "win32":
      return "pc";
    case "linux":
      return "unknown";
  }
}

function system() {
  switch (os.platform()) {
    case "darwin":
      return "darwin";
    case "win32":
      return "windows";
    case "linux":
      return "linux";
  }
}

function environment() {
  switch (os.platform()) {
    case "win32":
      return "msvc";
    case "linux":
      switch (os.arch()) {
        case "arm":
          return "gnueabihf";
        default:
          return "gnu";
      }
  }
}

const version = JSON.parse(
  fs.readFileSync(path.resolve(__dirname, "package.json"))
)["version"];

const triple = [architecture(), vendor(), system(), environment()]
  .filter(Boolean)
  .join("-");

https.get(
  `https://downloads.litho.dev/litho-cli/v${version}/litho-cli-v${version}-${triple}.gz`,
  (response) => {
    if (response.statusCode !== 200) {
      throw `Received unexpected status code while downloading litho-cli (${response.statusCode}).`;
    }

    const stream = fs.createWriteStream(path.resolve(__dirname, "bin/litho"), {
      mode: 0o755,
    });
    const unzip = zlib.createGunzip();
    unzip.pipe(stream);
    response.pipe(unzip);
  }
);
