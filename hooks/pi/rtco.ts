// RTCO Pi extension — rewrites bash commands to use rtco for token savings.
// Requires: rtco >= 0.23.0 in PATH.
//
// This is a thin delegating extension: all rewrite logic lives in `rtco rewrite`,
// which is the single source of truth (src/discover/registry.rs).
// To add or change rewrite rules, edit the Rust registry — not this file.
//
// Exit code contract for `rtco rewrite`:
//   0 + stdout  Rewrite found → mutate command
//   1           No RTCO equivalent → pass through unchanged
//   3 + stdout  Rewrite (advisory) → mutate command

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent"
import { isToolCallEventType } from "@earendil-works/pi-coding-agent"

const REWRITE_TIMEOUT_MS = 2_000
const MIN_SUPPORTED_RTK_MINOR = 23

// Parse "X.Y.Z" semver, return [major, minor, patch] or null.
function parseSemver(raw: string): [number, number, number] | null {
  const m = raw.trim().match(/(\d+)\.(\d+)\.(\d+)/)
  if (!m) return null
  return [parseInt(m[1], 10), parseInt(m[2], 10), parseInt(m[3], 10)]
}

// Calls `rtco rewrite`; returns the rewritten command or null (pass through).
async function rewriteCommand(
  pi: ExtensionAPI,
  cmd: string,
  signal?: AbortSignal
): Promise<string | null> {
  const result = await pi.exec("rtco", ["rewrite", cmd], {
    timeout: REWRITE_TIMEOUT_MS,
    signal,
  })
  if (result.killed) return null
  if (result.code !== 0 && result.code !== 3) return null
  return result.stdout.trim() || null
}

export default async function (pi: ExtensionAPI) {
  // Probe rtco version at load time; disables extension if missing or too old.
  const ver = await pi.exec("rtco", ["--version"], { timeout: REWRITE_TIMEOUT_MS })
  if (ver.code !== 0) {
    console.warn("[rtco] rtco binary not found in PATH — extension disabled")
    return
  }

  // Warn and bail if rtco predates 0.23.0 (when `rtco rewrite` was introduced).
  const parsed = parseSemver(ver.stdout.replace(/^rtco\s+/, ""))
  if (parsed) {
    const [major, minor] = parsed
    if (major === 0 && minor < MIN_SUPPORTED_RTK_MINOR) {
      console.warn(`[rtco] rtco ${ver.stdout.trim()} is too old (need >= 0.23.0) — extension disabled`)
      return
    }
  }

  pi.on("tool_call", async (event, ctx) => {
    try {
      if (!isToolCallEventType("bash", event)) return

      const cmd = event.input.command
      if (typeof cmd !== "string" || cmd.trim() === "") return

      if (cmd.startsWith("rtco ")) return
      if (process.env.RTK_DISABLED === "1") return

      // Delegate to RTCO.
      const rewritten = await rewriteCommand(pi, cmd, ctx.signal)
      if (rewritten && rewritten !== cmd) {
        event.input.command = rewritten
      }
    } catch (err) {
      // Fail open: never block execution on an unexpected error.
      console.warn("[rtco] unexpected error in tool_call handler; passing through command", err)
      return
    }
  })
}
