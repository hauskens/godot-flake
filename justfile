# Build the Godot export templates into a GC-rooted out-link (survives nix-collect-garbage)
build-templates:
    nix build .#godot-export-templates -o .export-templates/result

# Build + link templates into Godot's data dir so the editor's Export dialog finds them
export-templates: build-templates
    #!/usr/bin/env bash
    set -euo pipefail
    root="$(pwd)/.export-templates/result/share/godot/export_templates"
    dst="${XDG_DATA_HOME:-$HOME/.local/share}/godot/export_templates"
    mkdir -p "$dst"
    for v in "$root"/*/; do
      ln -sfn "$v" "$dst/$(basename "$v")"
    done
    echo "Linked Godot export templates: $(ls "$dst")"
