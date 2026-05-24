---
model: haiku
description: RTCO environment diagnostics - Checks installation, hooks, version, command routing
---

# /diagnose

Vérifie l'état de l'environnement RTCO et suggère des corrections.

## Quand utiliser

- **Automatiquement suggéré** quand Claude détecte ces patterns d'erreur :
  - `rtco: command not found` → RTCO non installé ou pas dans PATH
  - Hook errors in Claude Code → Hooks mal configurés ou non exécutables
  - `Unknown command` dans RTCO → Version incompatible ou commande non supportée
  - Token savings reports missing → `rtco gain` not working
  - Command routing errors → Hook integration broken

- **Manuellement** après installation, mise à jour RTCO, ou si comportement suspect

## Exécution

### 1. Vérifications parallèles

Lancer ces commandes en parallèle :

```bash
# RTCO installation check
which rtco && rtco --version || echo "❌ RTCO not found in PATH"
```

```bash
# Git status (verify working directory)
git status --short && git branch --show-current
```

```bash
# Hook configuration check
if [ -f ".claude/hooks/rtco-rewrite.sh" ]; then
    echo "✅ OK: rtco-rewrite.sh hook present"
    # Check if hook is executable
    if [ -x ".claude/hooks/rtco-rewrite.sh" ]; then
        echo "✅ OK: hook is executable"
    else
        echo "⚠️ WARNING: hook not executable (chmod +x needed)"
    fi
else
    echo "❌ MISSING: rtco-rewrite.sh hook"
fi
```

```bash
# Hook rtco-suggest.sh check
if [ -f ".claude/hooks/rtco-suggest.sh" ]; then
    echo "✅ OK: rtco-suggest.sh hook present"
    if [ -x ".claude/hooks/rtco-suggest.sh" ]; then
        echo "✅ OK: hook is executable"
    else
        echo "⚠️ WARNING: hook not executable (chmod +x needed)"
    fi
else
    echo "❌ MISSING: rtco-suggest.sh hook"
fi
```

```bash
# Claude Code context check
if [ -n "$CLAUDE_CODE_HOOK_BASH_TEMPLATE" ]; then
    echo "✅ OK: Running in Claude Code context"
    echo "   Hook env var set: CLAUDE_CODE_HOOK_BASH_TEMPLATE"
else
    echo "⚠️ WARNING: Not running in Claude Code (hooks won't activate)"
    echo "   CLAUDE_CODE_HOOK_BASH_TEMPLATE not set"
fi
```

```bash
# Test command routing (dry-run)
if command -v rtco >/dev/null 2>&1; then
    # Test if rtco gain works (validates install)
    if rtco --help | grep -q "gain"; then
        echo "✅ OK: rtco gain available"
    else
        echo "❌ MISSING: rtco gain command (old version or wrong binary)"
    fi
else
    echo "❌ RTCO binary not found"
fi
```

### 2. Validate token analytics

```bash
# Run rtco gain to verify analytics work
if command -v rtco >/dev/null 2>&1; then
    echo ""
    echo "📊 Token Savings (last 5 commands):"
    rtco gain --history 2>&1 | head -8 || echo "⚠️ rtco gain failed"
else
    echo "⚠️ Cannot test rtco gain (binary not installed)"
fi
```

### 3. Quality checks (if in RTCO repo)

```bash
# Only run if we're in RTCO repository
if [ -f "Cargo.toml" ] && grep -q 'name = "rtco"' Cargo.toml 2>/dev/null; then
    echo ""
    echo "🦀 RTCO Repository Quality Checks:"

    # Check if cargo fmt passes
    if cargo fmt --all --check >/dev/null 2>&1; then
        echo "✅ OK: cargo fmt (code formatted)"
    else
        echo "⚠️ WARNING: cargo fmt needed"
    fi

    # Check if cargo clippy would pass (don't run full check, just verify binary)
    if command -v cargo-clippy >/dev/null 2>&1 || cargo clippy --version >/dev/null 2>&1; then
        echo "✅ OK: cargo clippy available"
    else
        echo "⚠️ WARNING: cargo clippy not installed"
    fi
else
    echo "ℹ️ Not in RTCO repository (skipping quality checks)"
fi
```

## Format de sortie

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔍 RTCO Environment Diagnostic
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 RTCO Binary:      ✅ OK (v0.16.0) | ❌ NOT FOUND
🔗 Hooks:           ✅ OK (rtco-rewrite.sh + rtco-suggest.sh executable)
                    ❌ MISSING or ⚠️ WARNING (not executable)
📊 Token Analytics: ✅ OK (rtco gain working)
                    ❌ FAILED (command not available)
🎯 Claude Context:  ✅ OK (hook environment detected)
                    ⚠️ WARNING (not in Claude Code)
🦀 Code Quality:    ✅ OK (fmt + clippy ready) [if in RTCO repo]
                    ⚠️ WARNING (needs formatting/clippy)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Actions suggérées

Utiliser `AskUserQuestion` si problèmes détectés :

```
question: "Problèmes détectés. Quelles corrections appliquer ?"
header: "Fixes"
multiSelect: true
options:
  - label: "cargo install --path ."
    description: "Installer RTCO localement depuis le repo"
  - label: "chmod +x .claude/hooks/bash/*.sh"
    description: "Rendre les hooks exécutables"
  - label: "Tout corriger (recommandé)"
    description: "Install RTCO + fix hooks permissions"
```

**Adaptations selon contexte** :

### Si RTCO non installé
```
options:
  - label: "cargo install --path ."
    description: "Installer RTCO localement (si dans le repo)"
  - label: "cargo install rtco"
    description: "Installer RTCO depuis crates.io (dernière release)"
  - label: "brew install rtco-ai/tap/rtco"
    description: "Installer RTCO via Homebrew (macOS/Linux)"
```

### Si hooks manquants/non exécutables
```
options:
  - label: "chmod +x .claude/hooks/*.sh"
    description: "Rendre tous les hooks exécutables"
  - label: "Copier hooks depuis template"
    description: "Si hooks manquants, copier depuis repository principal"
```

### Si rtco gain échoue
```
options:
  - label: "Réinstaller RTCO"
    description: "cargo install --path . --force (version outdated?)"
  - label: "Vérifier version"
    description: "rtco --version (besoin v0.16.0+ pour rtco gain)"
```

## Exécution des fixes

### Fix 1 : Installer RTCO localement
```bash
# Depuis la racine du repo RTCO
cargo install --path .
# Vérifier installation
which rtco && rtco --version
```

### Fix 2 : Rendre hooks exécutables
```bash
chmod +x .claude/hooks/*.sh
# Vérifier permissions
ls -la .claude/hooks/*.sh
```

### Fix 3 : Tout corriger (recommandé)
```bash
# Install RTCO
cargo install --path .

# Fix hooks permissions
chmod +x .claude/hooks/*.sh

# Verify
which rtco && rtco --version && rtco gain --history | head -3
```

## Détection automatique

**IMPORTANT** : Claude doit suggérer `/diagnose` automatiquement quand il voit :

| Erreur | Pattern | Cause probable |
|--------|---------|----------------|
| RTCO not found | `rtco: command not found` | Pas installé ou pas dans PATH |
| Hook error | Hook execution failed, permission denied | Hooks non exécutables (`chmod +x` needed) |
| Version mismatch | `Unknown command` in RTCO output | Version RTCO incompatible (upgrade needed) |
| No analytics | `rtco gain` fails or command not found | RTCO install incomplete or old version |
| Command not rewritten | Commands not proxied via RTCO | Hook integration broken (check `CLAUDE_CODE_HOOK_BASH_TEMPLATE`) |

### Exemples de suggestion automatique

**Cas 1 : RTCO command not found**
```
Cette erreur "rtco: command not found" indique que RTCO n'est pas installé
ou pas dans le PATH. Je suggère de lancer `/diagnose` pour vérifier
l'installation et obtenir les commandes de fix.
```

**Cas 2 : Hook permission denied**
```
L'erreur "Permission denied" sur le hook rtco-rewrite.sh indique que
les hooks ne sont pas exécutables. Lance `/diagnose` pour identifier
le problème et corriger les permissions avec `chmod +x`.
```

**Cas 3 : rtco gain unavailable**
```
La commande `rtco gain` échoue, ce qui suggère une version RTCO obsolète
ou une installation incomplète. `/diagnose` va vérifier la version et
suggérer une réinstallation si nécessaire.
```

## Troubleshooting Common Issues

### Issue : RTCO installed but not in PATH

**Symptom**: `cargo install --path .` succeeds but `which rtco` fails

**Diagnosis**:
```bash
# Check if binary installed in Cargo bin
ls -la ~/.cargo/bin/rtco

# Check if ~/.cargo/bin in PATH
echo $PATH | grep -q .cargo/bin && echo "✅ In PATH" || echo "❌ Not in PATH"
```

**Fix**:
```bash
# Add to ~/.zshrc or ~/.bashrc
export PATH="$HOME/.cargo/bin:$PATH"

# Reload shell
source ~/.zshrc  # or source ~/.bashrc
```

### Issue : Multiple RTCO binaries (name collision)

**Symptom**: `rtco gain` fails with "command not found" even though `rtco --version` works

**Diagnosis**:
```bash
# Check if wrong RTCO installed (reachingforthejack/rtco)
rtco --version
# Should show "rtco X.Y.Z", NOT "Rust Type Kit"

rtco --help | grep gain
# Should show "gain" command - if missing, wrong binary
```

**Fix**:
```bash
# Uninstall wrong RTCO
cargo uninstall rtco

# Install correct RTCO (this repo)
cargo install --path .

# Verify
rtco gain --help  # Should work
```

### Issue : Hooks not triggering in Claude Code

**Symptom**: Commands not rewritten to `rtco <cmd>` automatically

**Diagnosis**:
```bash
# Check if in Claude Code context
echo $CLAUDE_CODE_HOOK_BASH_TEMPLATE
# Should print hook template path - if empty, not in Claude Code

# Check hooks exist and executable
ls -la .claude/hooks/*.sh
# Should show -rwxr-xr-x (executable)
```

**Fix**:
```bash
# Make hooks executable
chmod +x .claude/hooks/*.sh

# Verify hooks load in new Claude Code session
# (restart Claude Code session after chmod)
```

## Version Compatibility Matrix

| RTCO Version | rtco gain | rtco discover | Python/Go support | Notes |
|-------------|----------|--------------|-------------------|-------|
| v0.14.x     | ❌ No    | ❌ No        | ❌ No             | Outdated, upgrade |
| v0.15.x     | ✅ Yes   | ❌ No        | ❌ No             | Missing discover |
| v0.16.x     | ✅ Yes   | ✅ Yes       | ✅ Yes            | **Recommended** |
| main branch | ✅ Yes   | ✅ Yes       | ✅ Yes            | Latest features |

**Upgrade recommendation**: If running v0.15.x or older, upgrade to v0.16.x:

```bash
# From the RTCO repo root
git pull origin main
cargo install --path . --force
rtco --version  # Should show 0.16.x or newer
```
