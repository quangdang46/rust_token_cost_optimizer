import type { Plugin } from "@opencode-ai/plugin"

// RTCO OpenCode plugin — rewrites commands to use rtco for token savings.
// Requires: rtco >= 0.23.0 in PATH.
//
// This is a thin delegating plugin: all rewrite logic lives in `rtco rewrite`,
// which is the single source of truth (src/discover/registry.rs).
// To add or change rewrite rules, edit the Rust registry — not this file.

const RtkOpenCodePlugin: Plugin = async ({ $ }) => {
  try {
    await $`which rtco`.quiet()
  } catch {
    console.warn("[rtco] rtco binary not found in PATH — plugin disabled")
    return {}
  }

  return {
    "tool.execute.before": async (input, output) => {
      const tool = String(input?.tool ?? "").toLowerCase()
      if (tool !== "bash" && tool !== "shell") return
      const args = output?.args
      if (!args || typeof args !== "object") return

      const command = (args as Record<string, unknown>).command
      if (typeof command !== "string" || !command) return

      try {
        const result = await $`rtco rewrite ${command}`.quiet().nothrow()
        const rewritten = String(result.stdout).trim()
        if (rewritten && rewritten !== command) {
          ;(args as Record<string, unknown>).command = rewritten
        }
      } catch {
        // rtco rewrite failed — pass through unchanged
      }
    },
  }
}


export default RtkOpenCodePlugin
