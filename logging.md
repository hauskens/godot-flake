# Godot 4.5+ → Grafana Loki Logging

A ready-to-use logging stack for Godot game development with Grafana Loki and a pre-built dashboard.

## Quick Start

### 1. Start the Stack

```bash
docker compose up -d
```

This starts:
- **Loki** on `http://localhost:3100` (log ingestion)
- **Grafana** on `http://localhost:3000` (visualization)

### 2. Add Logger to Your Godot Project

Copy `godot/loki_logger.gd` to your project.

### 3. Create an Autoload

Create a script (or use the example `game_manager.gd`):

```gdscript
extends Node

func _ready() -> void:
    LokiLogger.setup(self)

func _notification(what: int) -> void:
    LokiLogger.handle_notification(what)
```

Add it as an autoload in **Project > Project Settings > Globals > Autoload**.

### 4. Enable Backtraces in Release Builds

**Project Settings > Debug > Settings > GDScript > Always Track Call Stacks** → Enable

### 5. Start Logging

```gdscript
LokiLogger.info("Player spawned", {"name": "Hero", "level": 5})
LokiLogger.warn("Health low", {"hp": 10})
LokiLogger.error("Failed to load save")
LokiLogger.critical("Unrecoverable state")
```

### 6. View Logs in Grafana

Open http://localhost:3000 and go to **Dashboards > Godot Game Logs**.

Login: `admin` / `admin` (or anonymous access is enabled by default).

## Architecture

```
┌─────────────┐     HTTP POST      ┌──────────┐     Query      ┌─────────┐
│   Godot     │ ─────────────────► │   Loki   │ ◄───────────── │ Grafana │
│   Game      │   /loki/api/v1/push│  :3100   │                │  :3000  │
└─────────────┘                    └──────────┘                └─────────┘
     │                                  │
     │ Batched JSON logs                │ Stored in
     │ every 5s or 10 entries           │ /loki/chunks
     ▼                                  ▼
```

## Log Format

Logs are sent as JSON with these fields:

```json
{
  "level": "INFO",
  "message": "Player spawned",
  "source": "app",
  "name": "Hero",
  "level": 5
}
```

Labels attached to the stream:
- `app="godot"`
- `session_id="<unique-per-run>"`
- `debug="true|false"`

## Grafana Queries (LogQL)

```logql
# All logs
{app="godot"}

# Errors and criticals
{app="godot"} |~ "ERROR|CRITICAL"

# Parse JSON and filter by level
{app="godot"} | json | level="ERROR"

# Search message content
{app="godot"} | json | message=~".*player.*"

# Specific session
{app="godot", session_id="67890abc-1234"}

# Count errors in last hour
count_over_time({app="godot"} |= "ERROR" [1h])
```

## Configuration

Edit `loki_logger.gd`:

```gdscript
# Point to your Loki instance
const LOKI_URL: String = "http://localhost:3100/loki/api/v1/push"

# Batch settings
const BATCH_SIZE: int = 10        # Logs before auto-flush
const FLUSH_INTERVAL: float = 5.0  # Seconds between flushes

# Disable console output in release
LokiLogger.print_to_console = OS.is_debug_build()
```

## Production Deployment

Example with basic auth:

```gdscript
const LOKI_URL: String = "https://logs.yourgame.com/loki/api/v1/push"
const LOKI_USER: String = "gamedev"
const LOKI_PASS: String = "secret"

# In _send_batch():
var auth := Marshalls.utf8_to_base64("%s:%s" % [LOKI_USER, LOKI_PASS])
var headers := [
    "Content-Type: application/json",
    "Authorization: Basic " + auth
]
```

NOW=$(date +%s)
curl -X POST "http://localhost:3100/loki/api/v1/push" \
  -H "Content-Type: application/json" \
  -d '{
    "streams": [{
      "stream": { "app": "godot", "session_id": "test-session", "debug": "true" },
      "values": [
        ["'$NOW'000000001", "{\"level\":\"INFO\",\"message\":\"Game started\",\"source\":\"app\"}"],
        ["'$NOW'000000002", "{\"level\":\"INFO\",\"message\":\"Player spawned\",\"source\":\"app\",\"name\":\"Hero\"}"],
        ["'$NOW'000000003", "{\"level\":\"WARN\",\"message\":\"Low memory warning\",\"source\":\"app\"}"],
        ["'$NOW'000000004", "{\"level\":\"ERROR\",\"message\":\"Failed to load texture\",\"source\":\"app\",\"path\":\"res://missing.png\"}"],
        ["'$NOW'000000005", "{\"level\":\"CRITICAL\",\"message\":\"Save corruption detected\",\"source\":\"app\"}"]
      ]
    }]
  }