#!/bin/bash

# Простой тест установки Rono

set -e

echo "🧪 Тестирование установки Rono..."

# Проверка, что rono установлен
if ! command -v rono &> /dev/null; then
    echo "❌ rono не найден в PATH"
    exit 1
fi

echo "✅ rono найден в PATH"

# Проверка версии
echo "📋 Версия: $(rono --version 2>/dev/null || echo 'unknown')"

# Создание тестового файла
cat > test_hello.rono << 'EOF'
chif main() {
    con.out("Hello from Rono!");
    con.out("Installation test successful!");
}
EOF

echo "🚀 Запуск тестовой программы..."

# Запуск тестовой программы
if rono run test_hello.rono; then
    echo "✅ Тест прошел успешно!"
    rm -f test_hello.rono
else
    echo "❌ Тест не прошел"
    rm -f test_hello.rono
    exit 1
fi

echo "🎉 Установка Rono работает корректно!"