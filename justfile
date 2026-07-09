# Run a crate's godot-bevy integration tests headlessly. The cdylib must be rebuilt
# with the `itest` feature *before* Godot loads it — `cargo run` alone only rebuilds
# the launcher, not the .so. Each crate has a single bin, so `cargo run` selects it.
# Usage: `just itest` (template) or `just itest jam-test/rust`.
itest crate="games/template/rust":
    # `ulimit -c 0`: godot-bevy-test can trip a SIGSEGV during Godot's headless
    # shutdown (leaked ObjectDB instances) *after* tests finish; suppress the
    # multi-hundred-MB core dumps it would otherwise drop in the project dir.
    cd {{crate}} && ulimit -c 0 && cargo build --features itest --lib && cargo run --features itest

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
