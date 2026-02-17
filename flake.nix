{
  description = "Surge TUI - Terminal user interface for macOS Surge proxy management";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      let
        mkSurgeTui = { lang ? null, description ? "Terminal user interface for macOS Surge proxy management" }:
          pkgs.rustPlatform.buildRustPackage {
            pname = "surge-tui";
            version = "0.1.1";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            buildFeatures = if lang != null then [ lang ] else [];
            meta = with pkgs.lib; {
              inherit description;
              homepage = "https://github.com/keyp-dev/surge-cli";
              license = licenses.mit;
              platforms = platforms.darwin;
            };
          };
      in
      {
        # Default: en-us (cargo default)
        packages.default = mkSurgeTui {};

        # Explicit language builds
        packages.en-us = mkSurgeTui {};
        packages.zh-cn = mkSurgeTui {
          lang = "zh-cn";
          description = "Terminal user interface for macOS Surge proxy management (中文)";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            clippy
            rustfmt
          ];
        };
      }
    );
}
