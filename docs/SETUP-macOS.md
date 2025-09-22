# Claude Code Schedule – macOS System Daemon Setup (launchd + zsh)

This guide shows how to run Claude Code Schedule on macOS as a system-level LaunchDaemon (starts at boot, no login required). It also includes a user-level LaunchAgent alternative.

The app keeps a persistent loop and triggers runs at: 07:00, 12:00, 17:00, 22:00, 03:00 local time. In ping mode it sends the global weather query (heavy token usage) identical to Linux.

## 0) Requirements
- macOS 12+ (Intel or Apple Silicon)
- Rust toolchain (rustup)
- Claude CLI installed (Homebrew or npm)
  - Apple Silicon (preferred): `brew install claude-code-cli` → `/opt/homebrew/bin/claude`
  - Intel (preferred): `brew install claude-code-cli` → `/usr/local/bin/claude`
  - npm global: `npm i -g @anthropic-ai/claude-code` (ensure the binary is on disk; you can set `CLAUDE_BIN`)

## 1) Build the binary
```bash
cd ~/githubTools/claude-code-schedule
# (optional) switch to the mac branch
# git checkout devMac

cargo build --release
# Binary will be at:
#   ~/githubTools/claude-code-schedule/target/release/ccschedule
```

## 2) Choose install mode (recommend: system daemon)
- System LaunchDaemon (boot start, no login): install the file below to `/Library/LaunchDaemons/`
  - `launchd/com.claude-code-schedule.daemon.plist`
- User LaunchAgent (start on login): install the file below to `~/Library/LaunchAgents/`
  - `launchd/com.claude-code-schedule.agent.plist`

Do NOT load both at the same time.

## 3) Configure paths (replace placeholders)
Edit the plist you picked and replace `REPLACE_WITH_USERNAME` with your macOS username. Also review these keys:
- `ProgramArguments`
  - Path to the built `ccschedule` binary
  - Include `--loop-mode --ping-mode` and set `--log-dir` (macOS convention: `~/Library/Logs/claude-code-schedule`)
- `WorkingDirectory`
  - Root of this repository on your Mac
- `EnvironmentVariables`
  - `CLAUDE_BIN`: strongly recommended to set explicitly:
    - Apple Silicon (Homebrew): `/opt/homebrew/bin/claude`
    - Intel (Homebrew): `/usr/local/bin/claude`
    - npm global: point to the actual `claude` binary you installed
  - `PATH`: set a minimal, predictable path for launchd (example below)

Example values (Apple Silicon):
```xml
<key>EnvironmentVariables</key>
<dict>
  <key>CLAUDE_BIN</key>
  <string>/opt/homebrew/bin/claude</string>
  <key>PATH</key>
  <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin</string>
</dict>
```

Tip (quick replace username):
```bash
# Apple Silicon example
sed -i '' "s|REPLACE_WITH_USERNAME|$USER|g" launchd/com.claude-code-schedule.daemon.plist
sed -i '' "s|REPLACE_WITH_USERNAME|$USER|g" launchd/com.claude-code-schedule.agent.plist
```

## 4) Install as a system daemon (recommended)
```bash
# Create system log dir for stdout/err
sudo mkdir -p /Library/Logs/claude-code-schedule
sudo chown root:wheel /Library/Logs/claude-code-schedule

# Install daemon plist
sudo cp launchd/com.claude-code-schedule.daemon.plist \
  /Library/LaunchDaemons/com.claude-code.schedule.plist
sudo chown root:wheel /Library/LaunchDaemons/com.claude-code.schedule.plist
sudo chmod 644 /Library/LaunchDaemons/com.claude-code.schedule.plist

# Load + enable (RunAtLoad + KeepAlive are in the plist)
sudo launchctl load -w /Library/LaunchDaemons/com.claude-code.schedule.plist

# Verify
sudo launchctl list | grep claude-code || true
```

If you previously used the user agent, unload it first to avoid duplicates:
```bash
launchctl unload -w ~/Library/LaunchAgents/com.claude-code.schedule.agent.plist 2>/dev/null || true
```

## 5) Install as a user agent (alternative)
```bash
mkdir -p ~/Library/LaunchAgents ~/Library/Logs/claude-code-schedule
cp launchd/com.claude-code-schedule.agent.plist \
  ~/Library/LaunchAgents/com.claude-code.schedule.agent.plist
launchctl load -w ~/Library/LaunchAgents/com.claude-code.schedule.agent.plist

# Verify
launchctl list | grep claude-code || true
```

## 6) What the daemon does
- The app runs continuously and manages its own schedule internally (07:00, 12:00, 17:00, 22:00, 03:00).
- It writes structured JSON logs to the `--log-dir` you configured (recommend: `~/Library/Logs/claude-code-schedule`).
- The launchd stdout/err logs are separate (see plist `StandardOutPath` / `StandardErrorPath`).

## 7) Check status and logs
```bash
# Daemon (system):
sudo launchctl print system/com.claude-code.schedule 2>/dev/null | head -n 50

# Agent (user):
launchctl print gui/$(id -u)/com.claude-code.schedule 2>/dev/null | head -n 50

# App JSON logs (weather responses included):
tail -f ~/Library/Logs/claude-code-schedule/$(date +%F).log
```

## 8) Quick one-shot test (without launchd)
```bash
cd ~/githubTools/claude-code-schedule
CLAUDE_BIN=/opt/homebrew/bin/claude \
  ./target/release/ccschedule \
  --ping-mode \
  --time "$(date -v+1M '+%H:%M')" \
  --log-dir ~/Library/Logs/claude-code-schedule
```
You should see a JSON line with `"status":"success"` and a long `response_content` (the global weather analysis). That confirms the CLI path and tokens flow are good.

## 9) Troubleshooting (zsh/launchd specifics)
- "Failed to execute claude command":
  - Your `CLAUDE_BIN` is wrong or not set. Update the plist `EnvironmentVariables` and reload.
- No logs appear:
  - Ensure `--log-dir` exists and is writable for the running user.
  - Confirm you edited `REPLACE_WITH_USERNAME` and absolute paths correctly.
- Daemon not listed:
  - `sudo launchctl list | grep com.claude-code.schedule`
  - If missing, try: `sudo launchctl unload -w /Library/LaunchDaemons/com.claude-code.schedule.plist && sudo launchctl load -w /Library/LaunchDaemons/com.claude-code.schedule.plist`
- Timezone:
  - The app uses `chrono::Local`; it follows your macOS system timezone automatically.
- Remove/uninstall:
  ```bash
  sudo launchctl unload -w /Library/LaunchDaemons/com.claude-code.schedule.plist
  sudo rm /Library/LaunchDaemons/com.claude-code.schedule.plist
  ```

## 10) Notes on parity with Linux
- Scheduling, weather query, and token consumption are identical by design.
- Systemd ⇄ launchd difference is abstracted away; the app manages its own schedule.
- If you ever need to run the full Claude command (not ping mode), remove `--ping-mode` from `ProgramArguments` in the plist and reload.

