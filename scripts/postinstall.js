#!/usr/bin/env node
const { ensureBinary } = require('./common');

ensureBinary((message) => console.log(message)).catch((error) => {
  console.warn(`[hashit] ${error.message}`);
  process.exit(0);
});
