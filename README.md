# 🚀 Rono Programming Language

[![Release](https://img.shields.io/github/v/release/EvgeniiAndronov/Rono)](https://github.com/EvgeniiAndronov/Rono/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/EvgeniiAndronov/Rono/workflows/Release/badge.svg)](https://github.com/EvgeniiAndronov/Rono/actions)

Rono - современный интерпретируемый язык программирования с поддержкой структур, указателей, модулей и многого другого.

## ✨ Возможности

- 🔢 **Базовые типы данных**: int, float, bool, str, nil
- 🏗️ **Структуры и методы**: объектно-ориентированное программирование
- 📚 **Массивы и списки**: с встроенными методами (.len(), .add(), .del())
- 🔄 **Управляющие конструкции**: for, while, if-else с поддержкой вложенности
- 👉 **Указатели и ссылки**: прямая работа с памятью
- 📦 **Система модулей**: импорт и использование внешних модулей
- 🛠️ **Встроенные функции**: консоль, HTTP запросы, генерация случайных чисел
- 🎨 **Строковая интерполяция**: удобная работа со строками

## 🔧 Установка

### Быстрая установка (рекомендуется)

**macOS/Linux:**
```bash
curl -sSL https://raw.githubusercontent.com/EvgeniiAndronov/Rono/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
iwr -useb https://raw.githubusercontent.com/EvgeniiAndronov/Rono/main/scripts/install.ps1 | iex
```

**Windows (альтернативный способ):**
```cmd
curl -L https://raw.githubusercontent.com/EvgeniiAndronov/Rono/main/scripts/install-windows.cmd -o install.cmd && install.cmd
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
# или
paru -S rono-lang
```

**Cargo (Rust):**
```bash
cargo install rono-lang
```

### Из исходного кода

```bash
git clone https://github.com/EvgeniiAndronov/Rono.git
cd Rono
cargo build --release
sudo cp target/release/rono /usr/local/bin/
```

### Скачать бинарные файлы

Скачайте готовые бинарные файлы для вашей платформы со страницы [Releases](https://github.com/EvgeniiAndronov/Rono/releases).

## 🎯 Быстрый старт

### Hello World

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

### Структуры и методы

```rono
struct Point {
    x: int,
    y: int,
}

fn_for Point {
    fn distance_from_origin(self) int {
        ret self.x * self.x + self.y * self.y;
    }
    
    fn move_by(self, dx: int, dy: int) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }
}

chif main() {
    var point: Point = Point { x = 3, y = 4 };
    con.out("Distance: {point.distance_from_origin()}");
    
    point.move_by(1, 1);
    con.out("New position: ({point.x}, {point.y})");
}
```

### Списки и циклы

```rono
chif main() {
    list numbers: int[] = [1, 2, 3, 4, 5];
    
    numbers.add(6);
    numbers.addAt(0, 0);
    
    con.out("Список содержит {numbers.len()} элементов");
    
    for (i = 0; i < numbers.len(); i = i + 1) {
        con.out("Элемент {i}: {numbers[i]}");
    }
}
```

### Модули

**math_utils.rono:**
```rono
fn add(a: int, b: int) int {
    ret a + b;
}

fn multiply(a: int, b: int) int {
    ret a * b;
}
```

**main.rono:**
```rono
import "math_utils";

chif main() {
    var result: int = add(5, 3);
    con.out("5 + 3 = {result}");
    
    var product: int = multiply(4, 7);
    con.out("4 * 7 = {product}");
}
```

## 🎨 Поддержка редакторов

Rono поддерживает подсветку синтаксиса в популярных редакторах:

- **VS Code**: Полная поддержка с сниппетами и автодополнением
- **Sublime Text**: Подсветка синтаксиса
- **Vim/Neovim**: Подсветка синтаксиса
- **Emacs**: Базовая поддержка
- **Atom**: Подсветка синтаксиса

[📖 Инструкции по установке](editor-support/INSTALLATION.md)

## 📚 Документация

- [Руководство по развертыванию](DEPLOYMENT_GUIDE.md)
- [Поддержка редакторов](editor-support/INSTALLATION.md)
- [Примеры кода](examples/)
- [Тесты интерпретатора](interpreter_test_suite/)

## 🧪 Тестирование

Запуск всех тестов:
```bash
cargo test
```

Запуск тестов интерпретатора:
```bash
rono run interpreter_test_suite/run_all_tests.rono
```

Запуск отдельных тестов:
```bash
rono run interpreter_test_suite/01_basic_types.rono
rono run interpreter_test_suite/02_structs_methods.rono
rono run interpreter_test_suite/03_arrays_lists.rono
# и так далее...
```

## 🛠️ Разработка

### Требования

- Rust 1.70+
- Cargo

### Сборка

```bash
git clone https://github.com/yourusername/rono-lang.git
cd rono-lang
cargo build
```

### Запуск в режиме разработки

```bash
cargo run -- run examples/hello.rono
```

## 🤝 Участие в разработке

Мы приветствуем вклад в развитие Rono! Пожалуйста:

1. Форкните репозиторий
2. Создайте ветку для вашей функции (`git checkout -b feature/amazing-feature`)
3. Зафиксируйте изменения (`git commit -m 'Add amazing feature'`)
4. Отправьте в ветку (`git push origin feature/amazing-feature`)
5. Откройте Pull Request

## 📄 Лицензия

Этот проект лицензирован под лицензией MIT - см. файл [LICENSE](LICENSE) для деталей.

## 🙏 Благодарности

- Rust сообществу за отличные инструменты
- Cranelift за JIT компиляцию
- Всем контрибьюторам проекта

## 📞 Поддержка

- 🐛 [Сообщить об ошибке](https://github.com/EvgeniiAndronov/Rono/issues)
- 💡 [Предложить функцию](https://github.com/EvgeniiAndronov/Rono/issues)
- 💬 [Обсуждения](https://github.com/EvgeniiAndronov/Rono/discussions)

---

**Сделано с ❤️ для сообщества разработчиков**