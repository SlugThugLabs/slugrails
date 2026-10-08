#!/usr/bin/env node
const { spawnSync } = require('child_process');
const path = require('path');
const os = require('os');
const fs = require('fs');

const home = process.env.SLUGTHUGLABS_HOME || process.env.SLUGTHUG_HOME || path.join(os.homedir(), '.slugthuglabs');
const isWin = os.platform() === 'win32';
const exeName = isWin ? 'slugrails.exe' : 'slugrails';
const binaryPath = path.join(home, 'bin', exeName);

const args = process.argv.slice(2);

if (fs.existsSync(binaryPath)) {
  const res = spawnSync(binaryPath, args, { stdio: 'inherit' });
  process.exit(res.status ?? 0);
} else {
  console.log(`SlugRails binary not found at ${binaryPath}.`);
  console.log("Please install via 'cargo install --path .' or download a pre-built binary from GitHub Releases:");
  console.log("https://github.com/SlugThugLabs/slugrails-mcp/releases");
  process.exit(1);
}
