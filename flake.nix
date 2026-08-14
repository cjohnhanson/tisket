{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      crane,
      rust-overlay,
      ...
    }:
    let
      forEachSystem =
        f:
        nixpkgs.lib.genAttrs
          [
            "aarch64-darwin"
            "x86_64-linux"
            "aarch64-linux"
            "x86_64-darwin"
          ]
          (
            system:
            f {
              pkgs = import nixpkgs {
                inherit system;
                overlays = [ rust-overlay.overlays.default ];
              };
              inherit system;
            }
          );
    in
    {
      packages = forEachSystem (
        { pkgs, system }:
        let
          toolchain = pkgs.rust-bin.stable.latest.default;
          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

          # cleanCargoSource strips non-Rust files; the bundled docs and
          # missouri fixtures must survive the filter.
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter =
              path: type:
              (craneLib.filterCargoSources path type)
              || (builtins.match ".*tests/missouri/.*" path != null)
              || (builtins.match ".*\\.yml$" path != null)
              || (builtins.match ".*\\.missouri.*" path != null)
              || (builtins.match ".*/docs$" path != null)
              || (builtins.match ".*/docs/.*" path != null)
              || (builtins.match ".*/skills$" path != null)
              || (builtins.match ".*/skills/.*" path != null);
          };

          commonArgs = {
            pname = "tisket";
            version = "0.1.0";
            inherit src;
            strictDeps = true;
            buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          tool = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              # Unit tests only in the nix check; the missouri suite runs in
              # development (it needs the missouri binary, which lives in its
              # own derivation).
              checkPhase = ''
                tmpHome="$(mktemp -d)"
                export HOME="$tmpHome"
                cargo test --profile release --locked
              '';
              # Man pages and shell completions come from the built
              # binary itself, so they always match the real CLI.
              postInstall = ''
                mkdir -p $out/share/man/man1
                $out/bin/tisket gen-man $out/share/man/man1
                mkdir -p $out/share/zsh/site-functions
                mkdir -p $out/share/bash-completion/completions
                mkdir -p $out/share/fish/vendor_completions.d
                $out/bin/tisket gen-completions zsh > $out/share/zsh/site-functions/_tisket
                $out/bin/tisket gen-completions bash > $out/share/bash-completion/completions/tisket
                $out/bin/tisket gen-completions fish > $out/share/fish/vendor_completions.d/tisket.fish
              '';
            }
          );
        in
        {
          default = tool;
          tisket = tool;
        }
      );

      devShells = forEachSystem (
        { pkgs, ... }:
        {
          default = pkgs.mkShell {
            buildInputs = [
              pkgs.rust-bin.stable.latest.default
              pkgs.jq
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
          };
        }
      );
    };
}
