class Rono < Formula
  desc "Rono - современный интерпретируемый язык программирования"
  homepage "https://github.com/yourusername/rono-lang"
  url "https://github.com/yourusername/rono-lang/archive/v1.0.0.tar.gz"
  sha256 "YOUR_SHA256_HASH_HERE"
  license "MIT"
  version "1.0.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
    
    # Установка примеров и документации
    pkgshare.install "examples"
    pkgshare.install "interpreter_test_suite"
    doc.install "README.md"
    doc.install "DEPLOYMENT_GUIDE.md"
  end

  test do
    # Создание тестового файла
    (testpath/"hello.rono").write <<~EOS
      chif main() {
          con.out("Hello, Homebrew!");
      }
    EOS
    
    # Проверка версии
    assert_match version.to_s, shell_output("#{bin}/rono --version")
    
    # Проверка выполнения программы
    output = shell_output("#{bin}/rono run #{testpath}/hello.rono")
    assert_match "Hello, Homebrew!", output
  end
end