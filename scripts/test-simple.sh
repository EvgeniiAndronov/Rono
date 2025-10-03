#!/bin/bash

# Простой тест установки Rono без сложных зависимостей

echo "🧪 Простой тест установки Rono..."

# Проверка, что rono установлен
if ! command -v rono &> /dev/null; then
    echo "❌ rono не найден в PATH"
    echo "💡 Убедитесь, что /usr/local/bin в вашем PATH"
    echo "💡 Или перезапустите терминал"
    exit 1
fi

echo "✅ rono найден в PATH"

# Создание простого тестового файла
cat > test_simple.rono << 'EOF'
chif main() {
    con.out("Hello from Rono!");
    var x: int = 42;
    con.out("Test number: {x}");
}
EOF

echo "🚀 Запуск простого теста..."

# Запуск тестовой программы
if rono run test_simple.rono; then
    echo "✅ Простой тест прошел успешно!"
    rm -f test_simple.rono
else
    echo "❌ Простой тест не прошел"
    rm -f test_simple.rono
    exit 1
fi

echo "🎉 Установка Rono работает корректно!"