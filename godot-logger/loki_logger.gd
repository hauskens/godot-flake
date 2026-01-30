# loki_logger.gd
# Godot 4.5+ Logger that pushes to Grafana Loki
#
# Setup:
# 1. Add this script to your project
# 2. Create an autoload that calls LokiLogger.setup(self) in _ready()
# 3. Configure LOKI_URL to point to your Loki instance
#
# Usage:
#   LokiLogger.info("Player spawned")
#   LokiLogger.warn("Low health")
#   LokiLogger.error("Failed to load save")
#   LokiLogger.critical("Unrecoverable state")

class_name Log extends Logger

# ============================================================================
# Configuration
# ============================================================================

## Loki push endpoint - change this to your server
const LOKI_URL: String = "http://localhost:3100/loki/api/v1/push"

## How many log entries to batch before sending
const BATCH_SIZE: int = 10

## Max seconds between flushes (even if batch not full)
const FLUSH_INTERVAL: float = 5.0

## App label for Loki queries
const APP_LABEL: String = "godot"

## Also print to Godot console (disable in release for performance)
static var print_to_console: bool = true

# ============================================================================
# Log Levels
# ============================================================================

enum Level {
	DEBUG,
	INFO,
	WARN,
	ERROR,
	CRITICAL,
}

const LEVEL_STRINGS: PackedStringArray = ["DEBUG", "INFO", "WARN", "ERROR", "CRITICAL"]

const LEVEL_COLORS: Dictionary = {
	Level.DEBUG: "gray",
	Level.INFO: "lime_green",
	Level.WARN: "gold",
	Level.ERROR: "tomato",
	Level.CRITICAL: "crimson",
}

# ============================================================================
# Internal State
# ============================================================================

static var _buffer: Array[Dictionary] = []
static var _mutex := Mutex.new()
static var _http: HTTPRequest
static var _timer: Timer
static var _session_id: String
static var _is_setup: bool = false

# ============================================================================
# Initialization
# ============================================================================


static func _static_init() -> void:
	_session_id = _generate_session_id()
	OS.add_logger(Log.new())


## Call this from an autoload's _ready() to enable HTTP transport
static func setup(parent: Node) -> void:
	if _is_setup:
		return

	_http = HTTPRequest.new()
	_http.name = "LokiLoggerHTTP"
	_http.request_completed.connect(_on_request_completed)
	parent.add_child(_http)

	_timer = Timer.new()
	_timer.name = "LokiLoggerTimer"
	_timer.wait_time = FLUSH_INTERVAL
	_timer.timeout.connect(flush)
	_timer.autostart = true
	parent.add_child(_timer)

	_is_setup = true
	info("LokiLogger initialized | session=%s | debug=%s" % [_session_id, OS.is_debug_build()])


static func _generate_session_id() -> String:
	# Short unique ID for this game session
	var time := Time.get_unix_time_from_system()
	var rand := randi() % 0xFFFF
	return "%x-%04x" % [int(time) & 0xFFFFFFFF, rand]


# ============================================================================
# Virtual Overrides (intercept engine logs)
# ============================================================================


func _log_error(
	function: String,
	file: String,
	line: int,
	code: String,
	rationale: String,
	_editor_notify: bool,
	error_type: int,
	script_backtraces: Array[ScriptBacktrace]
) -> void:
	var level := Level.WARN if error_type == ERROR_TYPE_WARNING else Level.ERROR
	var message := rationale if rationale else code
	var backtrace := _get_backtrace_string(script_backtraces)

	_queue_entry(
		{
			"level": LEVEL_STRINGS[level],
			"message": message,
			"source": "engine",
			"file": file,
			"line": line,
			"function": function,
			"code": code,
			"backtrace": backtrace,
		}
	)


func _log_message(message: String, _error: bool) -> void:
	# Skip our own prints (marked with special tag)
	if message.begins_with("[lang=tlh]"):
		return

	var level := Level.ERROR if error else Level.INFO
	_queue_entry(
		{
			"level": LEVEL_STRINGS[level],
			"message": message.strip_edges(),
			"source": "print",
		}
	)


# ============================================================================
# Public API
# ============================================================================


static func debug(message: String, context: Dictionary = {}) -> void:
	_log(Level.DEBUG, message, context)


static func info(message: String, context: Dictionary = {}) -> void:
	_log(Level.INFO, message, context)


static func warn(message: String, context: Dictionary = {}) -> void:
	_log(Level.WARN, message, context)


static func error(message: String, context: Dictionary = {}) -> void:
	var backtrace := Engine.capture_script_backtraces()
	context["backtrace"] = _get_backtrace_string(backtrace)
	_log(Level.ERROR, message, context)


static func critical(message: String, context: Dictionary = {}) -> void:
	var backtrace := Engine.capture_script_backtraces()
	context["backtrace"] = _get_backtrace_string(backtrace)
	_log(Level.CRITICAL, message, context)
	# Always flush immediately on critical
	flush()


## Force send all buffered logs now
static func flush() -> void:
	_mutex.lock()
	_send_batch()
	_mutex.unlock()


# ============================================================================
# Internal Implementation
# ============================================================================


static func _log(level: Level, message: String, context: Dictionary) -> void:
	var entry := {
		"level": LEVEL_STRINGS[level],
		"message": message,
		"source": "app",
	}
	entry.merge(context)

	_queue_entry(entry)

	if print_to_console:
		_print_to_console(level, message, context)


static func _queue_entry(entry: Dictionary) -> void:
	entry["ts"] = Time.get_unix_time_from_system()

	_mutex.lock()
	_buffer.append(entry)
	if _buffer.size() >= BATCH_SIZE:
		_send_batch()
	_mutex.unlock()


static func _send_batch() -> void:
	if _buffer.is_empty():
		return

	if not _http or not _is_setup:
		# Not set up yet, just clear buffer to avoid memory growth
		_buffer.clear()
		return

	# Group logs by level to create separate streams
	var streams_by_level: Dictionary = {}

	for entry in _buffer:
		var level: String = entry.get("level", "INFO")
		if not streams_by_level.has(level):
			streams_by_level[level] = []

		var ts_ns := str(int(entry["ts"] * 1_000_000_000))
		entry.erase("ts")
		streams_by_level[level].append([ts_ns, JSON.stringify(entry)])

	# Build streams array with level labels
	var streams: Array = []
	for level in streams_by_level:
		streams.append({
			"stream": {
				"app": APP_LABEL,
				"session_id": _session_id,
				"debug": str(OS.is_debug_build()).to_lower(),
				"level": level,
			},
			"values": streams_by_level[level]
		})

	var payload := {"streams": streams}

	_buffer.clear()

	var json := JSON.stringify(payload)
	var headers := ["Content-Type: application/json"]

	# Fire and forget - don't block on response
	var err := _http.request(LOKI_URL, headers, HTTPClient.METHOD_POST, json)
	if err != OK:
		# Can't use push_error here (infinite loop), just print
		printerr("LokiLogger: Failed to send batch: ", err)


static func _on_request_completed(
	result: int, response_code: int, _headers: PackedStringArray, _body: PackedByteArray
) -> void:
	if result != HTTPRequest.RESULT_SUCCESS or response_code >= 400:
		printerr("LokiLogger: Push failed | result=%d code=%d" % [result, response_code])


static func _get_backtrace_string(backtraces: Array[ScriptBacktrace]) -> String:
	if backtraces.is_empty():
		return ""

	var gdscript_idx := backtraces.find_custom(
		func(bt: ScriptBacktrace) -> bool: return bt.get_language_name() == "GDScript"
	)

	if gdscript_idx == -1:
		return ""

	return str(backtraces[gdscript_idx])


static func _print_to_console(level: Level, message: String, context: Dictionary) -> void:
	var color: String = LEVEL_COLORS[level]
	var level_str: String = LEVEL_STRINGS[level]
	var time_str := Time.get_time_string_from_system()

	var line := "[%s] %s: %s" % [time_str, level_str, message]

	if context.has("backtrace") and not context["backtrace"].is_empty():
		line += "\n" + context["backtrace"]

	# Mark with [lang=tlh] so _log_message ignores it
	var formatted := "[lang=tlh][b][color=%s]%s[/color][/b][/lang]" % [color, line]
	print_rich.call_deferred(formatted)


# ============================================================================
# Cleanup hooks - call these from your main autoload
# ============================================================================


## Call from _notification() with relevant notifications
static func handle_notification(what: int) -> void:
	match what:
		Node.NOTIFICATION_WM_CLOSE_REQUEST, Node.NOTIFICATION_WM_GO_BACK_REQUEST, Node.NOTIFICATION_APPLICATION_FOCUS_OUT, Node.NOTIFICATION_APPLICATION_PAUSED:
			flush()
