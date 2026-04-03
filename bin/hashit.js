#!/usr/bin/env node
const { spawnSync } = require('node:child_process');
const { ensureBinary } = require('../scripts/common');

async function main() {
  const binary = await ensureBinary((message) => console.error(message));

  const result = spawnSync(binary, process.argv.slice(2), {
    stdio: 'inherit',
    shell: process.platform === 'win32',
  });

  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }

  process.exit(result.status ?? 0);
}

main().catch((error) => {
  console.error(`[hashit] ${error.message}`);
  process.exit(1);
});
