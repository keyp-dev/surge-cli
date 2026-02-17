class SurgeTui < Formula
  desc "Terminal user interface for macOS Surge proxy management"
  homepage "https://github.com/keyp-dev/surge-cli"
  url "https://github.com/keyp-dev/surge-cli/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "79d9b8ba3ad8eb2b97714d79bb611ce9f3933c225f23448f9a8c8ce135f57153"
  license "MIT"
  head "https://github.com/keyp-dev/surge-cli.git", branch: "main"

  depends_on "rust" => :build

  def install
    # Build with English as default
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      surge-tui requires Surge to be running with HTTP API enabled.

      Add to your Surge configuration:
        [General]
        http-api = your-secret-key@127.0.0.1:6171

      Then configure surge-tui:
        export SURGE_HTTP_API_KEY="your-secret-key"

      Or create ~/.config/surge-tui/surge-tui.toml:
        [surge]
        http_api_host = "127.0.0.1"
        http_api_port = 6171
        http_api_key = "your-secret-key"
    EOS
  end

  test do
    assert_match "surge-tui", shell_output("#{bin}/surge-tui --version")
  end
end
