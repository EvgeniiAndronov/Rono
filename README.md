# Rono Programming Language

Rono - это современный функциональный компилируемый язык программирования, написанный на Rust. Язык сочетает в себе простоту синтаксиса с мощными возможностями функционального программирования и модульной архитектурой.

## Особенности языка

- **Функциональная парадигма** - вдохновлена Rust
- **Статическая типизация** с выводом типов
- **Система владения** и работа с указателями
- **Встроенные коллекции** - массивы, списки, словари с мутабельными методами
- **Структуры и методы** с поддержкой `self`
- **Модульная система** с импортами и алиасами
- **Интерполяция строк** с поддержкой цепочки вызовов
- **Встроенные функции** для работы с консолью и генерации случайных значений
- **Указатели и ссылки** с настоящей мутацией

## Установка

```bash
git clone <repository-url>
cd chif-lang
cargo build --release
```

## Использование

### Компиляция файла
```bash
cargo run -- examples/bubble_sort.rono
```

### Компиляция и запуск
```bash
cargo run -- examples/bubble_sort.rono --run
```

## Синтаксис языка

### Основная функция
```rono
chif main() {
    // код программы
}
```

### Переменные и константы
```rono
let pi: float = 3.14;           // константа
var counter: int = 0;           // переменная
var name: str = nil;            // переменная с nil значением
```

### Типы данных
- `int` - целые числа
- `float` - числа с плавающей точкой
- `str` - строки
- `bool` - логические значения (true/false)
- `nil` - отсутствие значения

### Коллекции
```rono
// Массивы (фиксированный размер)
array numbers: int[3] = [1, 2, 3];

// Списки (динамический размер)
list names: str[] = ["Alice", "Bob"];

// Многомерные коллекции
array matrix: int[2][2] = [[1, 2], [3, 4]];
list grid: str[][] = [["a", "b"], ["c", "d"]];

// Словари
var book: map[str:int] = {"page1": 1, "page2": 2};
```

### Функции
```rono
fn add(a: int, b: int) int {
    ret a + b;
}

fn greet(name: str) nil {
    con.out("Hello, {name}!");
}
```

### Структуры
```rono
struct Person {
    name: str,
    age: int,
    email: str,
}

// Создание экземпляра
var person: Person = Person {
    name = "Alice",
    age = 30,
    email = "alice@example.com",
};
```

### Методы структур
```rono
fn_for Person {
    fn getInfo(self) str {
        ret "Name: {self.name}, Age: {self.age}";
    }
    
    fn setAge(self, new_age: int) nil {
        self.age = new_age;
    }
}

// Использование методов
person.setAge(31);
let info: str = person.getInfo();
```

### Условные операторы
```rono
if (age >= 18) {
    con.out("Adult");
} else {
    con.out("Minor");
}

// Сложные условия
if ((age >= 18) && (age < 65)) || (status == "student") {
    con.out("Eligible");
}
```

### Циклы
```rono
// For цикл
for (i = 0; i < 10; i + 1) {
    con.out("Iteration: {i}");
}

// While цикл
var counter: int = 0;
while (counter < 5) {
    con.out("Counter: {counter}");
    counter = counter + 1;
}
```

### Switch/Case
```rono
switch grade:
case "A" {
    con.out("Excellent!");
}
case "B" {
    con.out("Good job!");
}
case "C" {
    con.out("Average");
}
default {
    con.out("Needs improvement");
}
```

### Работа с консолью
```rono
// Вывод в консоль
con.out("Hello, World!");
con.out("Value: {variable}");

// Ввод из консоли
var input: str = nil;
con.in(*input);
```

### Встроенные методы коллекций
```rono
list numbers: int[] = [1, 2, 3, 4, 5];

let length: int = numbers.len();    // получить длину: 5
numbers.add(6);                     // добавить элемент в конец
numbers.addAt(0, 0);               // вставить 0 в позицию 0
numbers.del(2);                     // удалить элемент по индексу 2

con.out("Length: {numbers.len()}"); // Length: 6
con.out("First: {numbers[0]}");     // First: 0
```

### Генерация случайных значений
```rono
let random_int: int = randi(1, 100);        // случайное целое от 1 до 100
let random_float: float = randf(0.0, 1.0);  // случайное float от 0.0 до 1.0
let random_char: str = rands("a", "z");     // случайный символ от 'a' до 'z'
```

### Модульная система
```rono
// math_utils.rono
fn add(a: int, b: int) int {
    ret a + b;
}

fn multiply(a: int, b: int) int {
    ret a * b;
}

// main.rono
import "math_utils" as math;

chif main() {
    let sum: int = math.add(5, 3);        // 8
    let product: int = math.multiply(4, 7); // 28
    con.out("Sum: {sum}, Product: {product}");
}
```

### Импорт структур и методов
```rono
// person.rono
struct Person {
    name: str,
    age: int,
}

// person_methods.rono
import "person";

fn_for Person {
    fn greet(self) str {
        ret "Hello, I'm {self.name}!";
    }
    
    fn setAge(self, new_age: int) nil {
        self.age = new_age;
    }
}

// main.rono
import "person";
import "person_methods";

chif main() {
    var alice: Person = Person { name = "Alice", age = 25 };
    let greeting: str = alice.greet();
    con.out(greeting); // "Hello, I'm Alice!"
    
    alice.setAge(26);
    con.out("New age: {alice.age}"); // "New age: 26"
}
```

### Работа с указателями и мутацией
```rono
fn swap(a: pointer, b: pointer) nil {
    let temp: int = *a;
    a = *b;
    b = temp;
}

chif main() {
    var x: int = 10;
    var y: int = 20;
    
    con.out("Before: x = {x}, y = {y}");
    swap(&x, &y);
    con.out("After: x = {x}, y = {y}");
    // Выводит: Before: x = 10, y = 20
    //          After: x = 20, y = 10
}
```

## Примеры программ

### Сортировка пузырьком
```rono
fn sort_bubble(a: list[int]) list[int] {
    for (i = 0; i < a.len(); i + 1) {
        for (j = 1; j < a.len(); j + 1) {
            if (a[i] > a[j]) {
                let temp: int = a[i];
                a[i] = a[j];
                a[j] = temp;
            }
        }
    }
    ret a;
}

chif main() {
    list numbers: int[] = [64, 34, 25, 12, 22, 11, 90];
    list sorted: int[] = sort_bubble(numbers);
    
    for (i = 0; i < sorted.len(); i + 1) {
        con.out("{sorted[i]} ");
    }
}
```

### Работа со структурами
```rono
struct Rectangle {
    width: float,
    height: float,
}

fn_for Rectangle {
    fn area(self) float {
        ret self.width * self.height;
    }
    
    fn perimeter(self) float {
        ret 2.0 * (self.width + self.height);
    }
}

chif main() {
    var rect: Rectangle = Rectangle {
        width = 10.0,
        height = 5.0,
    };
    
    let area: float = rect.area();
    let perimeter: float = rect.perimeter();
    
    con.out("Area: {area}");
    con.out("Perimeter: {perimeter}");
}
```

## Тестирование

Запуск тестов:
```bash
cargo test
```

Тесты покрывают:
- Лексический анализ
- Синтаксический анализ
- Выполнение программ
- Обработку ошибок

## Архитектура компилятора

1. **Лексер** (`src/lexer.rs`) - разбивает исходный код на токены
2. **Парсер** (`src/parser.rs`) - строит абстрактное синтаксическое дерево (AST)
3. **Интерпретатор** (`src/interpreter.rs`) - выполняет программу
4. **Система типов** (`src/types.rs`) - определяет типы данных и значения
5. **Обработка ошибок** (`src/error.rs`) - централизованная обработка ошибок

## Ограничения текущей версии

- Упрощенная реализация указателей
- Базовая интерполяция строк
- Ограниченная поддержка методов коллекций
- Отсутствие оптимизации кода
- Нет системы модулей

## Планы развития

- [ ] Полная реализация системы владения
- [ ] Компиляция в машинный код
- [ ] Система модулей и пакетов
- [ ] Расширенная стандартная библиотека
- [ ] Поддержка многопоточности
- [ ] Интеграция с внешними библиотеками

## Вклад в проект

Мы приветствуем вклад в развитие языка! Пожалуйста:

1. Создайте форк репозитория
2. Создайте ветку для новой функции
3. Добавьте тесты для новой функциональности
4. Убедитесь, что все тесты проходят
5. Создайте pull request

## Лицензия

MIT License - см. файл LICENSE для подробностей.

## Новые возможности

### Расширенная интерполяция строк
```rono
struct Person {
    name: str,
    address: Address,
}

struct Address {
    city: str,
    country: str,
}

chif main() {
    var person: Person = Person {
        name = "Alice",
        address = Address { city = "New York", country = "USA" }
    };
    
    list numbers: int[] = [1, 2, 3];
    
    // Поддержка полей, цепочки вызовов, индексации и методов
    con.out("Name: {person.name}");                    // Name: Alice
    con.out("City: {person.address.city}");           // City: New York
    con.out("First number: {numbers[0]}");            // First number: 1
    con.out("List length: {numbers.len()}");          // List length: 3
}
```

### Циклы с обратным направлением
```rono
chif main() {
    // Обычный цикл
    for (i = 0; i < 5; i + 1) {
        con.out("Forward: {i}");
    }
    
    // Обратный цикл
    for (i = 10; i > 0; i - 2) {
        con.out("Backward: {i}");
    }
    // Выводит: Backward: 10, Backward: 8, Backward: 6, Backward: 4, Backward: 2
}
```

### Полная модульная система
```rono
// Импорт с алиасом
import "math_utils" as math;

// Импорт без алиаса
import "string_utils";

// Вызов функций из модулей
let result: int = math.add(5, 3);
let processed: str = string_utils.capitalize("hello");
```

## Текущий статус

**Готовность: 98%** 🚀

### ✅ Полностью реализовано:
- Все базовые конструкции языка
- Структуры с методами и цепочкой вызовов  
- Коллекции с мутабельными методами (add, addAt, del)
- Указатели с настоящей мутацией
- Модульная система с импортами и алиасами
- Расширенная интерполяция строк
- Консольный ввод/вывод
- Все операторы и циклы (включая обратные)

### 🔄 В разработке:
- Рекурсивные функции в модулях
- Сетевая библиотека

**Язык Rono готов для написания полноценных программ!**