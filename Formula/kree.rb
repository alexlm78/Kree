class Kree < Formula
  desc "A directory tree visualizer and fuzzy finder for the terminal"
  homepage "https://github.com/alexlm78/Kree"
  url "https://github.com/alexlm78/Kree/archive/refs/tags/v0.2.13.tar.gz"
  # sha256 "UPDATE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "kree_cli")

    # Generate and install man page
    man_output = Utils.safe_popen_read(bin/"kree", "--man")
    (man1/"kree.1").write man_output

    # Generate and install shell completions
    generate_completions_from_executable(bin/"kree", "--completions")
  end

  test do
    assert_match "directories", shell_output("#{bin}/kree -d 1")
  end
end
