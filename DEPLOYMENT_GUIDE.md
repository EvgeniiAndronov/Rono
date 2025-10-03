# 🚀 Rono Language Deployment Guide

Полное руководство по развертыванию интерпретатора языка Rono как отдельной системы.

## 📋 Содержание

1. [Подготовка к публикации](#подготовка-к-публикации)
2. [Создание GitHub репозитория](#создание-github-репозитория)
3. [Скрипты установки](#скрипты-установки)
4. [Пакетные менеджеры](#пакетные-менеджеры)
5. [CI/CD настройка](#cicd-настройка)
6. [Документация](#документация)

## 🔧 Подготовка к публикации

### 1. Структура проекта

Убедитесь, что проект имеет правильную структуру:

```
rono-lang/
├── src/                    # Исходный код интерпретатора
├── examples/              # Примеры кода на Rono
├── interpreter_test_suite/ # Тесты интерпретатора
├── docs/                  # Документация
├── scripts/               # Скрипты установки
├── Cargo.toml            # Конфигурация Rust
├── README.md             # Основная документация
├── LICENSE               # Лицензия
└── CHANGELOG.md          # История изменений
```

### 2. Обновите Cargo.toml

```toml
[package]
name = "rono-lang"
version = "1.0.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Rono - современный интерпретируемый язык программирования"
license = "MIT"
repository = "https://github.com/yourusername/rono-lang"
homepage = "https://github.com/yourusername/rono-lang"
documentation = "https://docs.rs/rono-lang"
keywords = ["programming-language", "interpreter", "rono"]
categories = ["development-tools"]
readme = "README.md"

[[bin]]
name = "rono"
path = "src/main.rs"

[dependencies]
# Ваши зависимости
```

## 📦 Создание GitHub репозитория

### 1. Инициализация репозитория

```bash
# В папке Rono
git init
git add .
git commit -m "Initial commit: Rono Language Interpreter v1.0.0"

# Создайте репозиторий на GitHub, затем:
git remote add origin https://github.com/yourusername/rono-lang.git
git branch -M main
git push -u origin main
```

### 2. Создайте теги для релизов

```bash
git tag -a v1.0.0 -m "Release v1.0.0: First stable release"
git push origin v1.0.0
```

## 🛠 Скрипты установки

### 1. Универсальный скрипт установки (install.sh)

Создайте файл `scripts/install.sh`:

```bash
#!/bin/bash

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Информация о проекте
REPO="yourusername/rono-lang"
BINARY_NAME="rono"
INSTALL_DIR="/usr/local/bin"

echo -e "${BLUE}🚀 Установка интерпретатора языка Rono${NC}"
echo "=================================================="

# Определение операционной системы
detect_os() {
    case "$(uname -s)" in
        Darwin*)    OS="macos" ;;
        Linux*)     OS="linux" ;;
        CYGWIN*|MINGW*|MSYS*) OS="windows" ;;
        *)          OS="unknown" ;;
    esac
}

# Определение архитектуры
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) ARCH="x86_64" ;;
        arm64|aarch64) ARCH="aarch64" ;;
        armv7l) ARCH="armv7" ;;
        *) ARCH="unknown" ;;
    esac
}

# Проверка зависимостей
check_dependencies() {
    echo -e "${YELLOW}📋 Проверка зависимостей...${NC}"
    
    # Проверка curl или wget
    if ! command -v curl &> /dev/null && ! command -v wget &> /dev/null; then
        echo -e "${RED}❌ Ошибка: curl или wget не найдены${NC}"
        exit 1
    fi
    
    # Проверка tar
    if ! command -v tar &> /dev/null; then
        echo -e "${RED}❌ Ошибка: tar не найден${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Все зависимости найдены${NC}"
}

# Загрузка бинарного файла
download_binary() {
    detect_os
    detect_arch
    
    if [ "$OS" = "unknown" ] || [ "$ARCH" = "unknown" ]; then
        echo -e "${RED}❌ Неподдерживаемая платформа: $OS $ARCH${NC}"
        echo -e "${YELLOW}💡 Попробуйте установить из исходного кода${NC}"
        exit 1
    fi
    
    # Получение последней версии
    echo -e "${YELLOW}🔍 Получение информации о последней версии...${NC}"
    
    if command -v curl &> /dev/null; then
        LATEST_VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        LATEST_VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    fi
    
    if [ -z "$LATEST_VERSION" ]; then
        echo -e "${RED}❌ Не удалось получить информацию о версии${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}📦 Последняя версия: $LATEST_VERSION${NC}"
    
    # Формирование URL для загрузки
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_VERSION/rono-$LATEST_VERSION-$OS-$ARCH.tar.gz"
    
    echo -e "${YELLOW}⬇️  Загрузка $BINARY_NAME...${NC}"
    
    # Создание временной директории
    TMP_DIR=$(mktemp -d)
    cd "$TMP_DIR"
    
    # Загрузка архива
    if command -v curl &> /dev/null; then
        curl -L "$DOWNLOAD_URL" -o "rono.tar.gz"
    else
        wget "$DOWNLOAD_URL" -O "rono.tar.gz"
    fi
    
    # Распаковка
    tar -xzf "rono.tar.gz"
    
    echo -e "${GREEN}✅ Загрузка завершена${NC}"
}

# Установка из исходного кода
install_from_source() {
    echo -e "${YELLOW}🔨 Установка из исходного кода...${NC}"
    
    # Проверка Rust
    if ! command -v cargo &> /dev/null; then
        echo -e "${YELLOW}📦 Установка Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
    
    # Клонирование репозитория
    TMP_DIR=$(mktemp -d)
    cd "$TMP_DIR"
    
    git clone "https://github.com/$REPO.git"
    cd "rono-lang"
    
    # Сборка
    echo -e "${YELLOW}🔨 Сборка проекта...${NC}"
    cargo build --release
    
    # Копирование бинарного файла
    cp "target/release/$BINARY_NAME" "$TMP_DIR/"
}

# Установка бинарного файла
install_binary() {
    echo -e "${YELLOW}📦 Установка $BINARY_NAME...${NC}"
    
    # Проверка прав доступа
    if [ ! -w "$INSTALL_DIR" ]; then
        echo -e "${YELLOW}🔐 Требуются права администратора для установки в $INSTALL_DIR${NC}"
        sudo cp "$BINARY_NAME" "$INSTALL_DIR/"
        sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
    else
        cp "$BINARY_NAME" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    echo -e "${GREEN}✅ $BINARY_NAME установлен в $INSTALL_DIR${NC}"
}

# Проверка установки
verify_installation() {
    echo -e "${YELLOW}🧪 Проверка установки...${NC}"
    
    if command -v "$BINARY_NAME" &> /dev/null; then
        VERSION=$($BINARY_NAME --version 2>/dev/null || echo "unknown")
        echo -e "${GREEN}✅ $BINARY_NAME успешно установлен!${NC}"
        echo -e "${GREEN}📋 Версия: $VERSION${NC}"
        echo ""
        echo -e "${BLUE}🎉 Готово! Теперь вы можете использовать команду '$BINARY_NAME'${NC}"
        echo -e "${BLUE}💡 Попробуйте: $BINARY_NAME --help${NC}"
    else
        echo -e "${RED}❌ Установка не удалась${NC}"
        exit 1
    fi
}

# Очистка временных файлов
cleanup() {
    if [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ]; then
        rm -rf "$TMP_DIR"
    fi
}

# Основная функция
main() {
    trap cleanup EXIT
    
    check_dependencies
    
    # Попытка загрузить бинарный файл, если не удается - сборка из исходного кода
    if download_binary 2>/dev/null; then
        echo -e "${GREEN}✅ Бинарный файл загружен${NC}"
    else
        echo -e "${YELLOW}⚠️  Бинарный файл недоступен, установка из исходного кода...${NC}"
        install_from_source
    fi
    
    install_binary
    verify_installation
}

# Запуск
main "$@"
```

### 2. Скрипт для Windows (install.ps1)

Создайте файл `scripts/install.ps1`:

```powershell
# Rono Language Installer for Windows
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\Rono",
    [switch]$AddToPath = $true
)

$ErrorActionPreference = "Stop"

# Цвета для вывода
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

Write-ColorOutput Blue "🚀 Установка интерпретатора языка Rono"
Write-Output "=================================================="

# Информация о проекте
$REPO = "yourusername/rono-lang"
$BINARY_NAME = "rono.exe"

# Создание директории установки
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Получение последней версии
Write-ColorOutput Yellow "🔍 Получение информации о последней версии..."
try {
    $LatestRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
    $Version = $LatestRelease.tag_name
    Write-ColorOutput Green "📦 Последняя версия: $Version"
} catch {
    Write-ColorOutput Red "❌ Не удалось получить информацию о версии"
    exit 1
}

# Загрузка бинарного файла
$DownloadUrl = "https://github.com/$REPO/releases/download/$Version/rono-$Version-windows-x86_64.zip"
$ZipPath = "$env:TEMP\rono.zip"

Write-ColorOutput Yellow "⬇️  Загрузка $BINARY_NAME..."
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath
    Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force
    Remove-Item $ZipPath
    Write-ColorOutput Green "✅ Загрузка завершена"
} catch {
    Write-ColorOutput Red "❌ Ошибка загрузки: $_"
    exit 1
}

# Добавление в PATH
if ($AddToPath) {
    Write-ColorOutput Yellow "🔧 Добавление в PATH..."
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($CurrentPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallDir", "User")
        Write-ColorOutput Green "✅ Добавлено в PATH"
    }
}

# Проверка установки
Write-ColorOutput Yellow "🧪 Проверка установки..."
$RonoPath = Join-Path $InstallDir $BINARY_NAME
if (Test-Path $RonoPath) {
    Write-ColorOutput Green "✅ $BINARY_NAME успешно установлен!"
    Write-ColorOutput Blue "🎉 Готово! Перезапустите терминал и используйте команду 'rono'"
    Write-ColorOutput Blue "💡 Попробуйте: rono --help"
} else {
    Write-ColorOutput Red "❌ Установка не удалась"
    exit 1
}
```

## 📦 Пакетные менеджеры

### 1. Homebrew (macOS/Linux)

Создайте файл `Formula/rono.rb`:

```ruby
class Rono < Formula
  desc "Rono - современный интерпретируемый язык программирования"
  homepage "https://github.com/yourusername/rono-lang"
  url "https://github.com/yourusername/rono-lang/archive/v1.0.0.tar.gz"
  sha256 "YOUR_SHA256_HASH"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/rono", "--version"
  end
end
```

**Процесс добавления в Homebrew:**

1. Создайте tap репозиторий:
```bash
# Создайте репозиторий homebrew-rono на GitHub
git clone https://github.com/yourusername/homebrew-rono.git
cd homebrew-rono
mkdir Formula
# Добавьте файл Formula/rono.rb
git add . && git commit -m "Add rono formula" && git push
```

2. Пользователи смогут установить:
```bash
brew tap yourusername/rono
brew install rono
```

### 2. APT (Debian/Ubuntu)

Создайте файл `debian/control`:

```
Source: rono-lang
Section: devel
Priority: optional
Maintainer: Your Name <your.email@example.com>
Build-Depends: debhelper (>= 10), cargo, rustc
Standards-Version: 4.1.2
Homepage: https://github.com/yourusername/rono-lang

Package: rono
Architecture: any
Depends: ${shlibs:Depends}, ${misc:Depends}
Description: Rono programming language interpreter
 Rono is a modern interpreted programming language with support for
 structures, pointers, modules, and more.
```

**Процесс создания .deb пакета:**

```bash
# Создайте скрипт build-deb.sh
#!/bin/bash
cargo build --release
mkdir -p debian/rono/usr/bin
cp target/release/rono debian/rono/usr/bin/
dpkg-deb --build debian/rono
```

### 3. AUR (Arch Linux)

Создайте файл `PKGBUILD`:

```bash
# Maintainer: Your Name <your.email@example.com>
pkgname=rono-lang
pkgver=1.0.0
pkgrel=1
pkgdesc="Rono programming language interpreter"
arch=('x86_64')
url="https://github.com/yourusername/rono-lang"
license=('MIT')
depends=()
makedepends=('cargo' 'rust')
source=("$pkgname-$pkgver.tar.gz::https://github.com/yourusername/rono-lang/archive/v$pkgver.tar.gz")
sha256sums=('YOUR_SHA256_HASH')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/rono" "$pkgdir/usr/bin/rono"
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
```

## 🔄 CI/CD настройка

### GitHub Actions

Создайте файл `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    name: Build and Release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact_name: rono
            asset_name: rono-linux-x86_64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: rono
            asset_name: rono-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact_name: rono
            asset_name: rono-macos-aarch64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: rono.exe
            asset_name: rono-windows-x86_64.exe

    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
        override: true
    
    - name: Build
      uses: actions-rs/cargo@v1
      with:
        command: build
        args: --release --target ${{ matrix.target }}
    
    - name: Create archive
      shell: bash
      run: |
        if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
          7z a ${{ matrix.asset_name }}.zip target/${{ matrix.target }}/release/${{ matrix.artifact_name }}
        else
          tar -czf ${{ matrix.asset_name }}.tar.gz -C target/${{ matrix.target }}/release ${{ matrix.artifact_name }}
        fi
    
    - name: Upload Release Asset
      uses: actions/upload-release-asset@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        upload_url: ${{ github.event.release.upload_url }}
        asset_path: ${{ matrix.asset_name }}.*
        asset_name: ${{ matrix.asset_name }}.*
        asset_content_type: application/octet-stream
```

## 📚 Документация

### 1. Обновите README.md

```markdown
# 🚀 Rono Programming Language

Современный интерпретируемый язык программирования с поддержкой структур, указателей, модулей и многого другого.

## 🔧 Установка

### Быстрая установка (рекомендуется)

**macOS/Linux:**
```bash
curl -sSL https://raw.githubusercontent.com/yourusername/rono-lang/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
iwr -useb https://raw.githubusercontent.com/yourusername/rono-lang/main/scripts/install.ps1 | iex
```

### Пакетные менеджеры

**Homebrew (macOS/Linux):**
```bash
brew tap yourusername/rono
brew install rono
```

**Arch Linux (AUR):**
```bash
yay -S rono-lang
```

### Из исходного кода

```bash
git clone https://github.com/yourusername/rono-lang.git
cd rono-lang
cargo build --release
sudo cp target/release/rono /usr/local/bin/
```

## 🎯 Быстрый старт

Создайте файл `hello.rono`:
```rono
chif main() {
    con.out("Hello, World!");
}
```

Запустите:
```bash
rono run hello.rono
```
```

### 2. Создайте CHANGELOG.md

```markdown
# Changelog

## [1.0.0] - 2024-01-XX

### Added
- Базовые типы данных (int, float, bool, str, nil)
- Структуры и методы
- Массивы и списки с методами
- Циклы и условия (for, while, if-else)
- Указатели и ссылки
- Система модулей и импортов
- Встроенные функции (консоль, HTTP, случайные числа)
- Строковая интерполяция
- Комплексный набор тестов

### Technical
- Интерпретатор на Rust с использованием Cranelift
- Поддержка macOS, Linux, Windows
- Скрипты автоматической установки
```

## 🎯 Пошаговый план развертывания

1. **Подготовка проекта:**
   ```bash
   cd Rono
   # Обновите Cargo.toml с правильными метаданными
   # Создайте LICENSE файл
   # Обновите README.md
   ```

2. **Создание GitHub репозитория:**
   ```bash
   git init
   git add .
   git commit -m "Initial release v1.0.0"
   # Создайте репозиторий на GitHub
   git remote add origin https://github.com/yourusername/rono-lang.git
   git push -u origin main
   ```

3. **Настройка CI/CD:**
   - Добавьте `.github/workflows/release.yml`
   - Создайте первый релиз через GitHub

4. **Создание скриптов установки:**
   - Добавьте `scripts/install.sh` и `scripts/install.ps1`
   - Протестируйте на разных платформах

5. **Настройка пакетных менеджеров:**
   - Создайте homebrew tap
   - Подготовьте PKGBUILD для AUR
   - Создайте .deb пакет

Этот план позволит вам создать полноценную систему распространения интерпретатора Rono! 🎉