#!/bin/bash

set -e

# Цвета для вывода
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Информация о проекте
BINARY_NAME="rono"
INSTALL_DIR="/usr/local/bin"

echo -e "${BLUE}🚀 Установка интерпретатора языка Rono (локальная версия)${NC}"
echo "=================================================="

# Проверка наличия релизной сборки
if [ ! -f "target/release/$BINARY_NAME" ]; then
    echo -e "${YELLOW}⚠️ Релизная сборка не найдена. Запускаю сборку...${NC}"
    cargo build --release
fi

# Копирование бинарного файла
echo -e "${YELLOW}📦 Копирование бинарного файла в $INSTALL_DIR...${NC}"
sudo cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo -e "${GREEN}✅ Установка завершена успешно!${NC}"
echo -e "${BLUE}🔍 Проверьте установку командой: $BINARY_NAME --version${NC}"
echo -e "${YELLOW}💡 Теперь вы можете запускать программы на языке Rono:${NC}"
echo -e "   $BINARY_NAME путь/к/вашей/программе.rono"