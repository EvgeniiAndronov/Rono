#!/bin/bash

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Информация о проекте
REPO="EvgeniiAndronov/Rono"
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
        echo -e "${YELLOW}💡 Установите curl: sudo apt install curl (Ubuntu) или brew install curl (macOS)${NC}"
        exit 1
    fi
    
    # Проверка tar
    if ! command -v tar &> /dev/null; then
        echo -e "${RED}❌ Ошибка: tar не найден${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Все зависимости найдены${NC}"
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
    
    # Проверка git
    if ! command -v git &> /dev/null; then
        echo -e "${RED}❌ Ошибка: git не найден${NC}"
        echo -e "${YELLOW}💡 Установите git для продолжения${NC}"
        exit 1
    fi
    
    # Клонирование репозитория
    TMP_DIR=$(mktemp -d)
    cd "$TMP_DIR"
    
    echo -e "${YELLOW}📥 Клонирование репозитория...${NC}"
    git clone "https://github.com/$REPO.git"
    cd "Rono"
    
    # Сборка
    echo -e "${YELLOW}🔨 Сборка проекта (это может занять несколько минут)...${NC}"
    cargo build --release
    
    # Копирование бинарного файла
    cp "target/release/$BINARY_NAME" "$TMP_DIR/"
    cd "$TMP_DIR"
}

# Установка бинарного файла
install_binary() {
    echo -e "${YELLOW}📦 Установка $BINARY_NAME в $INSTALL_DIR...${NC}"
    
    # Проверка существования файла
    if [ ! -f "$BINARY_NAME" ]; then
        echo -e "${RED}❌ Бинарный файл $BINARY_NAME не найден${NC}"
        exit 1
    fi
    
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
        VERSION=$($BINARY_NAME --version 2>/dev/null || echo "1.0.0")
        echo -e "${GREEN}✅ $BINARY_NAME успешно установлен!${NC}"
        echo -e "${GREEN}📋 Версия: $VERSION${NC}"
        echo ""
        echo -e "${BLUE}🎉 Готово! Теперь вы можете использовать команду '$BINARY_NAME'${NC}"
        echo -e "${BLUE}💡 Попробуйте: $BINARY_NAME --help${NC}"
        echo -e "${BLUE}📚 Документация: https://github.com/$REPO${NC}"
    else
        echo -e "${RED}❌ Установка не удалась${NC}"
        echo -e "${YELLOW}💡 Попробуйте добавить $INSTALL_DIR в PATH или перезапустите терминал${NC}"
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
    
    detect_os
    detect_arch
    
    echo -e "${BLUE}🖥️  Система: $OS $ARCH${NC}"
    
    check_dependencies
    install_from_source
    install_binary
    verify_installation
}

# Запуск
main "$@"# Force update
