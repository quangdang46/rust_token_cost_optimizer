#!/usr/bin/env bun
/**
 * Delete the RTCO test VM.
 * Usage: bun run scripts/benchmark/cleanup.ts
 */

import { vmDelete } from "./lib/vm";

console.log("Deleting rtco-test VM...");
await vmDelete();
console.log("Done.");
