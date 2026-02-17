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
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "surge-tui";
          version = "0.1.1";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildFeatures = [ "lang-en-us" ];
          meta = with pkgs.lib; {
            description = "Terminal user interface for macOS Surge proxy management";
            homepage = "https://github.com/keyp-dev/surge-cli";
            license = licenses.mit;
            platforms = platforms.darwin;
          };
        };

        packages.zh-cn = pkgs.rustPlatform.buildRustPackage {
          pname = "surge-tui";
          version = "0.1.1";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildFeatures = [ "lang-zh-cn" ];
          meta = with pkgs.lib; {
            description = "Terminal user interface for macOS Surge proxy management (Chinese)";
            homepage = "https://github.com/keyp-dev/surge-cli";
            license = licenses.mit;
            platforms = platforms.darwin;
          };
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
