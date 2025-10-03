#!/bin/bash

# Скрипт для быстрого развертывания Rono Language на GitHub

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🚀 Развертывание Rono Language${NC}"
echo "=================================="

# Проверка зависимостей
check_dependencies() {
    echo -e "${YELLOW}📋 Проверка зависимостей...${NC}"
    
    if ! command -v git &> /dev/null; then
        echo -e "${RED}❌ Git не найден${NC}"
        exit 1
    fi
    
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo не найден${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Все зависимости найдены${NC}"
}

# Проверка и обновление версии
update_version() {
    echo -e "${YELLOW}📝 Обновление версии...${NC}"
    
    # Получение текущей версии из Cargo.toml
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo -e "${BLUE}Текущая версия: $CURRENT_VERSION${NC}"
    
    read -p "Введите новую версию (или нажмите Enter для использования текущей): " NEW_VERSION
    
    if [ -n "$NEW_VERSION" ]; then
        # Обновление версии в Cargo.toml
        sed -i.bak "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
        rm Cargo.toml.bak 2>/dev/null || true
        
        # Обновление версии в README.md если есть
        if [ -f README.md ]; then
            sed -i.bak "s/v$CURRENT_VERSION/v$NEW_VERSION/g" README.md
            rm README.md.bak 2>/dev/null || true
        fi
        
        VERSION=$NEW_VERSION
        echo -e "${GREEN}✅ Версия обновлена до $VERSION${NC}"
    else
        VERSION=$CURRENT_VERSION
        echo -e "${BLUE}Используется текущая версия $VERSION${NC}"
    fi
}

# Сборка и тестирование
build_and_test() {
    echo -e "${YELLOW}🔨 Сборка проекта...${NC}"
    cargo build --release
    
    echo -e "${YELLOW}🧪 Запуск тестов...${NC}"
    cargo test
    
    echo -e "${YELLOW}🎯 Тестирование интерпретатора...${NC}"
    ./target/release/rono run interpreter_test_suite/01_basic_types.rono
    ./target/release/rono run interpreter_test_suite/02_structs_methods.rono
    ./target/release/rono run interpreter_test_suite/03_arrays_lists.rono
    
    echo -e "${GREEN}✅ Все тесты прошли успешно${NC}"
}

# Настройка Git репозитория
setup_git() {
    echo -e "${YELLOW}📦 Настройка Git репозитория...${NC}"
    
    # Проверка, инициализирован ли git
    if [ ! -d .git ]; then
        git init
        echo -e "${GREEN}✅ Git репозиторий инициализирован${NC}"
    fi
    
    # Добавление файлов
    git add .
    
    # Проверка изменений
    if git diff --staged --quiet; then
        echo -e "${YELLOW}⚠️  Нет изменений для коммита${NC}"
    else
        git commit -m "Release v$VERSION: Complete Rono Language implementation"
        echo -e "${GREEN}✅ Изменения зафиксированы${NC}"
    fi
}

# Создание тега
create_tag() {
    echo -e "${YELLOW}🏷️  Создание тега v$VERSION...${NC}"
    
    if git tag -l "v$VERSION" | grep -q "v$VERSION"; then
        echo -e "${YELLOW}⚠️  Тег v$VERSION уже существует${NC}"
        read -p "Удалить существующий тег? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git tag -d "v$VERSION"
            echo -e "${GREEN}✅ Старый тег удален${NC}"
        else
            echo -e "${YELLOW}Пропускаем создание тега${NC}"
            return
        fi
    fi
    
    git tag -a "v$VERSION" -m "Release v$VERSION: Rono Language Interpreter

🚀 Первый стабильный релиз интерпретатора языка Rono

✨ Возможности:
- Базовые типы данных (int, float, bool, str, nil)
- Структуры и методы
- Массивы и списки с методами
- Циклы и условия (for, while, if-else)
- Указатели и ссылки
- Система модулей и импортов
- Встроенные функции (консоль, HTTP, случайные числа)
- Строковая интерполяция

🛠️ Установка:
curl -sSL https://raw.githubusercontent.com/yourusername/rono-lang/main/scripts/install.sh | bash"

    echo -e "${GREEN}✅ Тег v$VERSION создан${NC}"
}

# Инструкции по публикации
show_publish_instructions() {
    echo ""
    echo -e "${BLUE}📋 Инструкции по публикации:${NC}"
    echo "=================================="
    echo ""
    echo -e "${YELLOW}1. Создайте репозиторий на GitHub:${NC}"
    echo "   - Перейдите на https://github.com/new"
    echo "   - Назовите репозиторий: rono-lang"
    echo "   - Сделайте его публичным"
    echo "   - НЕ инициализируйте с README, .gitignore или лицензией"
    echo ""
    echo -e "${YELLOW}2. Подключите локальный репозиторий:${NC}"
    echo "   git remote add origin https://github.com/yourusername/rono-lang.git"
    echo "   git branch -M main"
    echo ""
    echo -e "${YELLOW}3. Отправьте код на GitHub:${NC}"
    echo "   git push -u origin main"
    echo "   git push origin v$VERSION"
    echo ""
    echo -e "${YELLOW}4. Создайте релиз на GitHub:${NC}"
    echo "   - Перейдите на https://github.com/yourusername/rono-lang/releases"
    echo "   - Нажмите 'Create a new release'"
    echo "   - Выберите тег v$VERSION"
    echo "   - Заполните описание релиза"
    echo "   - Опубликуйте релиз"
    echo ""
    echo -e "${YELLOW}5. Настройте пакетные менеджеры:${NC}"
    echo "   - Homebrew: создайте tap репозиторий homebrew-rono"
    echo "   - AUR: отправьте PKGBUILD в AUR"
    echo "   - Crates.io: cargo publish"
    echo ""
    echo -e "${GREEN}🎉 После этого пользователи смогут устанавливать Rono!${NC}"
}

# Основная функция
main() {
    check_dependencies
    update_version
    build_and_test
    setup_git
    create_tag
    show_publish_instructions
    
    echo ""
    echo -e "${GREEN}🎉 Развертывание завершено успешно!${NC}"
    echo -e "${BLUE}Версия: v$VERSION${NC}"
    echo -e "${BLUE}Следуйте инструкциям выше для публикации на GitHub${NC}"
}

# Запуск
main "$@"