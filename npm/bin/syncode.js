#!/usr/bin/env node
"use strict";

const { execFileSync } = require("child_process");
const path = require("path");

const bin = path.join(__dirname, "syncode");

try {
  execFileSync(bin, process.argv.slice(2), { stdio: "inherit" });
} catch (err) {
  if (err.status !== null) {
    process.exit(err.status);
  }
  throw err;
}
