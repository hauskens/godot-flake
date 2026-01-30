# game_manager.gd
# Example autoload showing how to set up LokiLogger
#
# Add this as an autoload in Project Settings:
#   Project > Project Settings > Globals > Autoload
#   Path: res://game_manager.gd
#   Name: GameManager

extends Node


func _ready() -> void:
	# Initialize the HTTP transport for LokiLogger
	Log.setup(self)

	# Optionally disable console printing in release builds
	Log.print_to_console = OS.is_debug_build()

	# Example logs
	Log.info("Game started", {"version": "1.0.0"})


func _notification(what: int) -> void:
	# Ensure logs are flushed on important events
	Log.handle_notification(what)

# ============================================================================
# Example usage throughout your game:
# ============================================================================
#
# Log.debug("Spawning enemy", {"type": "goblin", "pos": position})
# Log.info("Level completed", {"level": 5, "time": 123.4})
# Log.warn("Low memory", {"available_mb": 128})
# Log.error("Failed to load resource", {"path": "res://missing.png"})
# Log.critical("Save corruption detected")
#
# ============================================================================
# Querying in Grafana:
# ============================================================================
#
# All logs:
#   {app="godot"}
#
# Errors only:
#   {app="godot"} |= "ERROR"
#
# Specific session:
#   {app="godot", session_id="12345678-abcd"}
#
# Parse JSON and filter:
#   {app="godot"} | json | level="ERROR"
#
# Search message content:
#   {app="godot"} | json | message=~".*player.*"
#
