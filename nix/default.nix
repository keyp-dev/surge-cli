{ lib
, rustPlatform
, fetchFromGitHub
}:

rustPlatform.buildRustPackage rec {
  pname = "surge-cli";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "keyp-dev";
    repo = "surge-cli";
    rev = "v${version}";
    sha256 = ""; # Will be calculated: nix-prefetch-url --unpack URL
  };

  cargoHash = ""; # Will be calculated automatically on first build

  # Build with English as default
  buildFeatures = [ "lang-en-us" ];

  meta = with lib; {
    description = "Terminal user interface for macOS Surge proxy management";
    homepage = "https://github.com/keyp-dev/surge-cli";
    license = licenses.mit;
    maintainers = with maintainers; [ ]; # Add your name
    platforms = platforms.darwin; # macOS only
  };
}
