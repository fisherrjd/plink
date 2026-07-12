{ pkgs ? import
    (fetchTarball {
      name = "jpetrucciani-2026-07-11";
      url = "https://github.com/jpetrucciani/nix/archive/50828b7a5cdc0a3da823d744a0033b68d27c4683.tar.gz";
      sha256 = "0p2jzvb5q7rrgfb8yamqv8zhqwm0l72j4fbyg4566b09cppm9zjf";
    })
    { }

}:
let
  name = "plink";

  # Official prebuilt editor: nixpkgs' godot_4 has no darwin binary cache and
  # building it from source takes hours (and hits linker crashes on arm64).
  godot-bin = pkgs.stdenv.mkDerivation rec {
    pname = "godot-bin";
    version = "4.7-stable";
    src = pkgs.fetchzip {
      url = "https://github.com/godotengine/godot/releases/download/${version}/Godot_v${version}_macos.universal.zip";
      sha256 = "sha256-99UWil62PMNTE0ohsc23Y3lH6QlYXxJkqcJP2QADMyk=";
      stripRoot = false;
    };
    dontFixup = true;
    installPhase = ''
      mkdir -p $out/Applications $out/bin
      cp -r Godot.app $out/Applications/
      printf '#!/bin/sh\nexec "%s/Applications/Godot.app/Contents/MacOS/Godot" "$@"\n' "$out" > $out/bin/godot
      chmod +x $out/bin/godot
    '';
  };

  tools = with pkgs; {
    cli = [
      jfmt
      nixup
    ];
    rust = [
      cargo
      rustc
      rust-analyzer
      clippy
      rustfmt
    ];
    game = [
      godot-bin
    ];
    scripts = pkgs.lib.attrsets.attrValues scripts;
  };

  scripts = with pkgs; {
    plink-build = writeShellScriptBin "plink-build" ''
      set -e
      cd "$(git rev-parse --show-toplevel)"
      cargo build --manifest-path rust/Cargo.toml "$@"
    '';
    plink-run = writeShellScriptBin "plink-run" ''
      set -e
      cd "$(git rev-parse --show-toplevel)"
      cargo build --manifest-path rust/Cargo.toml
      godot --path godot "$@"
    '';
    plink-edit = writeShellScriptBin "plink-edit" ''
      set -e
      cd "$(git rev-parse --show-toplevel)"
      cargo build --manifest-path rust/Cargo.toml
      godot --path godot --editor "$@"
    '';
  };
  paths = pkgs.lib.flatten [ (builtins.attrValues tools) ];
  env = pkgs.buildEnv {
    inherit name paths; buildInputs = paths;
  };
in
(env.overrideAttrs (_: {
  inherit name;
  NIXUP = "0.0.11";
})) // { inherit scripts; }
