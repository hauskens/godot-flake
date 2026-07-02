{
  description = "A flake using Oxalica's rust-overlay wrapped with bevy-flake.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    bevy-flake = {
      url = "github:swagtop/bevy-flake";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      bevy-flake,
      rust-overlay,
      ...
    }:
    bevy-flake.lib.mkFlake {
      perSystem =
        {
          pkgs,
          system,
          packages,
          formatter,
          ...
        }:
        let

          runtimeLibs = with pkgs; [
            vulkan-loader # libvulkan.so.1 — wgpu's Vulkan backend dlopens this
            libxkbcommon # required at runtime even on a pure-Wayland build
            wayland # libwayland-client.so — winit's Wayland backend
            alsa-lib # bevy_audio / rodio
            udev # gamepad + input device enumeration
            # X11 libs: only dlopen'd on the XWayland / WINIT_UNIX_BACKEND=x11 fallback.
            libx11
            libxcursor
            libxi
            libxrandr
            godotPackages_4_5.godot
          ];
        in
        {
          inherit packages formatter;

          devShells.default = pkgs.mkShell {
            name = "bevy-flake-rust-overlay";
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs =
              with pkgs;
              [
                # rustToolchain
                packages.rust-toolchain.develop
                clang # linker driver for -fuse-ld=mold
                mold # fast linker (wired in .cargo/config.toml)
                llvmPackages_latest.bintools # lld fallback + llvm-* tools
                vulkan-tools # vulkaninfo / vkcube to diagnose GPU/ICD issues
                glibc.dev # headers for bindgen (BINDGEN_EXTRA_CLANG_ARGS)
                glib.dev
              ]
              ++ runtimeLibs;

            shellHook = ''
              export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH"
              export LIBCLANG_PATH="${pkgs.llvmPackages_latest.libclang.lib}/lib"
              export BINDGEN_EXTRA_CLANG_ARGS="-I${pkgs.glibc.dev}/include -I${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include -I${pkgs.glib.dev}/include/glib-2.0 -I${pkgs.glib.out}/lib/glib-2.0/include/"

              # Deliberately NOT exporting RUSTFLAGS. Setting it would override the ENTIRE
              # .cargo/config.toml rustflags array (cargo reads flags from one source only,
              # with no merging), silently dropping the linker and -Zshare-generics flags.
              # All rustflags live in .cargo/config.toml instead.

              echo "Bevy 0.18 dev environment (NixOS/Wayland) ready."
            '';
            packages = [
              packages.rust-toolchain.develop
              packages.dioxus-cli.develop
              # packages.bevy-cli.develop
            ];
          };
        };

      config =
        {
          pkgs,
          system,
          ...
        }:
        {
          src = builtins.path {
            name = "src";
            path = ./.;

            # Ignore files that aren't needed in compilation of Bevy project.
            filter =
              path: type:
              !(builtins.elem (baseNameOf path) [
                "flake.lock"
                "flake.nix"
              ]);
          };

          rustToolchain =
            targets:
            let
              channel = "stable"; # For nightly, use "nightly".
            in
            pkgs.rust-bin.${channel}.latest.default.override {
              inherit targets;
              extensions = [
                "rust-src"
                "rust-analyzer"
                "clippy"
                "rustfmt"
                # "rustc-codegen-cranelift-preview"
              ];
            };

          withPkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
            config = {
              allowUnfree = true;
              microsoftVisualStudioLicenseAccepted = true;
            };
          };
        };
    };
}
#
#   outputs =
#     {
#       self,
#       nixpkgs,
#       treefmt-nix,
#       pre-commit-hooks,
#       rust-overlay,
#       ...
#     }@inputs:
#     let
#       forAllSystems = nixpkgs.lib.genAttrs [
#         "x86_64-linux"
#         #"aarch64-darwin"
#       ];
#       treefmtEval = forAllSystems (
#         system: treefmt-nix.lib.evalModule nixpkgs.legacyPackages.${system} ./treefmt.nix
#       );
#     in
#     {
#       formatter = forAllSystems (system: treefmtEval.${system}.config.build.wrapper);
#
#       checks = forAllSystems (system: {
#         formatting = treefmtEval.${system}.config.build.check self;
#         pre-commit-check = pre-commit-hooks.lib.${system}.run {
#           src = ./.;
#           hooks = {
#             treefmt.enable = true;
#             treefmt.package = treefmtEval.${system}.config.build.wrapper;
#           };
#         };
#       });
#
#       devShells = forAllSystems (
#         system:
#         let
#           overlays = [ (import rust-overlay) ];
#           pkgs = import nixpkgs {
#             inherit system overlays;
#           };
#
#           runtimeLibs = with pkgs; [
#             vulkan-loader # libvulkan.so.1 — wgpu's Vulkan backend dlopens this
#             libxkbcommon # required at runtime even on a pure-Wayland build
#             wayland # libwayland-client.so — winit's Wayland backend
#             alsa-lib # bevy_audio / rodio
#             udev # gamepad + input device enumeration
#             # X11 libs: only dlopen'd on the XWayland / WINIT_UNIX_BACKEND=x11 fallback.
#             libx11
#             libxcursor
#             libxi
#             libxrandr
#           ];
#
#         in
#         {
#           default = pkgs.mkShell {
#             nativeBuildInputs = with pkgs; [ pkg-config ];
#
#             buildInputs =
#               with pkgs;
#               [
#                 # rustToolchain
#                 rust-toolchain.develop
#                 clang # linker driver for -fuse-ld=mold
#                 mold # fast linker (wired in .cargo/config.toml)
#                 llvmPackages_latest.bintools # lld fallback + llvm-* tools
#                 vulkan-tools # vulkaninfo / vkcube to diagnose GPU/ICD issues
#                 glibc.dev # headers for bindgen (BINDGEN_EXTRA_CLANG_ARGS)
#                 glib.dev
#               ]
#               ++ runtimeLibs;
#
#             shellHook = ''
#               export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH"
#               export LIBCLANG_PATH="${pkgs.llvmPackages_latest.libclang.lib}/lib"
#               export BINDGEN_EXTRA_CLANG_ARGS="-I${pkgs.glibc.dev}/include -I${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include -I${pkgs.glib.dev}/include/glib-2.0 -I${pkgs.glib.out}/lib/glib-2.0/include/"
#
#               # Deliberately NOT exporting RUSTFLAGS. Setting it would override the ENTIRE
#               # .cargo/config.toml rustflags array (cargo reads flags from one source only,
#               # with no merging), silently dropping the linker and -Zshare-generics flags.
#               # All rustflags live in .cargo/config.toml instead.
#
#               echo "Bevy 0.18 dev environment (NixOS/Wayland) ready."
#             '';
#             packages = [
#               pkgs.rust-toolchain.develop
#               pkgs.dioxus-cli.develop
#               # packages.bevy-cli.develop
#             ];
#           };
#
#           #   packages = with pkgs; [
#           #     # godot_4
#           #     godotPackages_4_3.godot
#           #     rustc
#           #     cargo
#           #     rustfmt
#           #     rust-analyzer
#           #     pkg-config
#           #     clang
#           #   ];
#           #   RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
#           #   shellHook = ''
#           #     ${self.checks.${system}.pre-commit-check.shellHook}
#           #     echo "Welcome to the Godot + Rust dev shell"
#           #   '';
#           # };
#         }
#       );
#     };
# }
