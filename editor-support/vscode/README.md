# Rono Language Support for VS Code

Официальное расширение VS Code для языка программирования Rono.

## Возможности

- 🎨 **Подсветка синтаксиса** - полная поддержка всех конструкций языка Rono
- 📝 **Сниппеты кода** - быстрые шаблоны для основных конструкций
- 🔧 **Автодополнение скобок** - автоматическое закрытие скобок и кавычек
- 📁 **Сворачивание кода** - поддержка сворачивания блоков кода
- 💬 **Комментарии** - поддержка однострочных (//) и многострочных (/* */) комментариев

## Поддерживаемые конструкции

### Ключевые слова
- `chif`, `fn`, `struct`, `var`, `list`, `array`
- `if`, `else`, `for`, `while`, `break`, `continue`
- `ret`, `import`, `fn_for`
- `true`, `false`, `nil`, `self`

### Типы данных
- `int`, `float`, `bool`, `str`, `pointer`
- Пользовательские структуры

### Встроенные функции
- `con.out()`, `con.in()`
- `randi()`, `randf()`, `rands()`
- `http.get()`, `http.post()`
- `abs()`, `sqrt()`

### Операторы
- Арифметические: `+`, `-`, `*`, `/`, `%`
- Сравнения: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Логические: `&&`, `||`, `!`
- Указатели: `&`, `*`

## Сниппеты

Доступные сниппеты для быстрого кодирования:

- `main` - создать главную функцию
- `fn` - создать функцию
- `struct` - создать структуру
- `method` - создать метод структуры
- `if` - условный оператор
- `for` - цикл for
- `while` - цикл while
- `var` - объявление переменной
- `list` - создание списка
- `cout` - вывод в консоль
- `randi` - случайное целое число
- `randf` - случайное число с плавающей точкой
- `httpget` - HTTP GET запрос
- `listadd` - добавить элемент в список
- `import` - импорт модуля

## Пример кода

```rono
// Импорт модуля
import "math_utils";

// Структура
struct Point {
    x: int,
    y: int,
}

// Методы структуры
fn_for Point {
    fn distance(self) float {
        ret sqrt(self.x * self.x + self.y * self.y);
    }
}

// Главная функция
chif main() {
    var point: Point = Point { x = 3, y = 4 };
    con.out("Distance: {point.distance()}");
    
    list numbers: int[] = [1, 2, 3, 4, 5];
    numbers.add(6);
    
    for (i = 0; i < numbers.len(); i = i + 1) {
        con.out("Number: {numbers[i]}");
    }
    
    // Random numbers
    var random_num: int = randi(1, 100);
    con.out("Random: {random_num}");
}
```

## Установка

### Из VS Code Marketplace
1. Откройте VS Code
2. Перейдите в Extensions (Ctrl+Shift+X)
3. Найдите "Rono Language Support"
4. Нажмите Install

### Ручная установка
1. Скачайте `.vsix` файл из [релизов](https://github.com/EvgeniiAndronov/Rono/releases)
2. В VS Code: View → Command Palette → "Extensions: Install from VSIX"
3. Выберите скачанный файл

## Разработка

Для разработки расширения:

```bash
# Установите vsce
npm install -g @vscode/vsce

# Соберите расширение
vsce package

# Опубликуйте в Marketplace
vsce publish
```

## Поддержка

- 🐛 [Сообщить об ошибке](https://github.com/EvgeniiAndronov/Rono/issues)
- 💡 [Предложить улучшение](https://github.com/EvgeniiAndronov/Rono/issues)
- 📚 [Документация Rono](https://github.com/EvgeniiAndronov/Rono)

## Лицензия

MIT License - см. [LICENSE](https://github.com/EvgeniiAndronov/Rono/blob/main/LICENSE)