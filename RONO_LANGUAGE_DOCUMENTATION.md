# 📚 Полная документация языка программирования Rono

## 🎯 Содержание

1. [Введение](#введение)
2. [Установка и настройка](#установка-и-настройка)
3. [Базовый синтаксис](#базовый-синтаксис)
4. [Типы данных](#типы-данных)
5. [Переменные и константы](#переменные-и-константы)
6. [Операторы](#операторы)
7. [Управляющие конструкции](#управляющие-конструкции)
8. [Функции](#функции)
9. [Структуры и методы](#структуры-и-методы)
10. [Массивы и списки](#массивы-и-списки)
11. [Указатели и ссылки](#указатели-и-ссылки)
12. [Модули и импорты](#модули-и-импорты)
13. [Строковая интерполяция](#строковая-интерполяция)
14. [Встроенные функции](#встроенные-функции)
15. [Примеры программ](#примеры-программ)
16. [Лучшие практики](#лучшие-практики)
17. [Справочник по ошибкам](#справочник-по-ошибкам)

---

## 🌟 Введение

**Rono** — современный интерпретируемый язык программирования, разработанный для простоты изучения и эффективности разработки. Язык сочетает в себе простоту синтаксиса с мощными возможностями, включая поддержку структур, указателей, модулей и встроенных функций.

### Ключевые особенности:
- 🔢 **Статическая типизация** с выводом типов
- 🏗️ **Объектно-ориентированное программирование** через структуры и методы
- 📚 **Динамические коллекции** (списки) и статические массивы
- 👉 **Указатели** для прямой работы с памятью
- 📦 **Модульная система** для организации кода
- 🎨 **Строковая интерполяция** для удобной работы со строками
- 🛠️ **Богатый набор встроенных функций**

### Философия языка:
- **Простота**: Минимальный синтаксис для максимальной выразительности
- **Безопасность**: Статическая типизация предотвращает многие ошибки
- **Производительность**: JIT-компиляция через Cranelift
- **Читаемость**: Код должен быть понятен без комментариев

---

## 🔧 Установка и настройка

### Быстрая установка

**macOS/Linux:**
```bash
curl -sSL https://raw.githubusercontent.com/EvgeniiAndronov/Rono/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
iwr -useb https://raw.githubusercontent.com/EvgeniiAndronov/Rono/main/scripts/install.ps1 | iex
```

### Проверка установки
```bash
rono --version
```

### Запуск программы
```bash
rono run program.rono
```

---

## 📝 Базовый синтаксис

### Структура программы
Каждая программа на Rono должна содержать функцию `main`:

```rono
chif main() {
    con.out("Hello, World!");
}
```

### Комментарии
```rono
// Однострочный комментарий

/*
   Многострочный
   комментарий
*/
```

### Точка с запятой
Точка с запятой в конце строки **обязательна**:
```rono
var x: int = 10;
con.out("Значение x: {x}");
```

---

## 🔢 Типы данных

### Базовые типы

| Тип | Описание | Пример |
|-----|----------|--------|
| `int` | Целое число (64-бит) | `42`, `-17`, `0` |
| `float` | Число с плавающей точкой (64-бит) | `3.14`, `-2.5`, `0.0` |
| `bool` | Логическое значение | `true`, `false` |
| `str` | Строка | `"Hello"`, `"Мир"` |
| `nil` | Отсутствие значения | `nil` |

### Примеры объявления:
```rono
var age: int = 25;
var pi: float = 3.14159;
var is_active: bool = true;
var name: str = "Алексей";
var empty_value: int = nil;
```

### Автоматический вывод типов:
```rono
let count = 10;        // int
let price = 99.99;     // float
let active = true;     // bool
let message = "Hi";    // str
```

---

## 📊 Переменные и константы

### Объявление переменных
```rono
// Изменяемая переменная
var counter: int = 0;
counter = counter + 1;

// Неизменяемая переменная (константа)
let max_size: int = 100;
// max_size = 200; // Ошибка!
```

### Область видимости
```rono
chif main() {
    var global_var: int = 10;
    
    if (true) {
        var local_var: int = 20;
        con.out("Локальная: {local_var}");
        con.out("Глобальная: {global_var}");
    }
    
    // con.out("{local_var}"); // Ошибка! Переменная не видна
}
```

---

## ⚡ Операторы

### Арифметические операторы
```rono
var a: int = 10;
var b: int = 3;

var sum: int = a + b;        // 13
var diff: int = a - b;       // 7
var product: int = a * b;    // 30
var quotient: int = a / b;   // 3
var remainder: int = a % b;  // 1
```

### Операторы сравнения
```rono
var x: int = 5;
var y: int = 10;

var equal: bool = x == y;        // false
var not_equal: bool = x != y;    // true
var less: bool = x < y;          // true
var greater: bool = x > y;       // false
var less_equal: bool = x <= y;   // true
var greater_equal: bool = x >= y; // false
```

### Логические операторы
```rono
var a: bool = true;
var b: bool = false;

var and_result: bool = a && b;   // false
var or_result: bool = a || b;    // true
var not_result: bool = !a;       // false
```

### Приоритет операторов (от высшего к низшему):
1. `!` (логическое НЕ)
2. `*`, `/`, `%` (умножение, деление, остаток)
3. `+`, `-` (сложение, вычитание)
4. `<`, `<=`, `>`, `>=` (сравнение)
5. `==`, `!=` (равенство)
6. `&&` (логическое И)
7. `||` (логическое ИЛИ)

---

## 🔄 Управляющие конструкции

### Условные операторы

#### Простое условие:
```rono
var age: int = 18;

if (age >= 18) {
    con.out("Совершеннолетний");
}
```

#### Условие с альтернативой:
```rono
var score: int = 85;

if (score >= 90) {
    con.out("Отлично!");
} else {
    con.out("Хорошо!");
}
```

#### Множественные условия:
```rono
var grade: int = 75;

if (grade >= 90) {
    con.out("A");
} else {
    if (grade >= 80) {
        con.out("B");
    } else {
        if (grade >= 70) {
            con.out("C");
        } else {
            con.out("D");
        }
    }
}
```

### Циклы

#### Цикл for:
```rono
// Простой цикл
for (i = 1; i <= 5; i = i + 1) {
    con.out("Итерация: {i}");
}

// Цикл с шагом
for (i = 0; i < 10; i = i + 2) {
    con.out("Четное число: {i}");
}

// Обратный цикл
for (i = 10; i > 0; i = i - 1) {
    con.out("Обратный отсчет: {i}");
}
```

#### Цикл while:
```rono
var counter: int = 5;

while (counter > 0) {
    con.out("Счетчик: {counter}");
    counter = counter - 1;
}
```

#### Вложенные циклы:
```rono
// Таблица умножения
for (i = 1; i <= 3; i = i + 1) {
    for (j = 1; j <= 3; j = j + 1) {
        var result: int = i * j;
        con.out("{i} x {j} = {result}");
    }
}
```

---

## 🔧 Функции

### Объявление функций
```rono
// Функция без параметров и возвращаемого значения
fn greet() {
    con.out("Привет!");
}

// Функция с параметрами
fn greet_person(name: str) {
    con.out("Привет, {name}!");
}

// Функция с возвращаемым значением
fn add(a: int, b: int) int {
    ret a + b;
}

// Функция с несколькими параметрами и возвращаемым значением
fn calculate_area(width: float, height: float) float {
    ret width * height;
}
```

### Вызов функций
```rono
chif main() {
    greet();
    greet_person("Алексей");
    
    var sum: int = add(5, 3);
    con.out("Сумма: {sum}");
    
    var area: float = calculate_area(10.5, 7.2);
    con.out("Площадь: {area}");
}
```

### Рекурсивные функции
```rono
fn factorial(n: int) int {
    if (n <= 1) {
        ret 1;
    } else {
        var prev: int = factorial(n - 1);
        ret n * prev;
    }
}

chif main() {
    var result: int = factorial(5);
    con.out("5! = {result}"); // 120
}
```

---

## 🏗️ Структуры и методы

### Определение структур
```rono
struct Point {
    x: int,
    y: int,
}

struct Person {
    name: str,
    age: int,
    email: str,
}

struct Rectangle {
    top_left: Point,
    width: int,
    height: int,
}
```

### Создание экземпляров
```rono
// Простая структура
var point: Point = Point {
    x = 10,
    y = 20,
};

// Структура с вложенными структурами
var rect: Rectangle = Rectangle {
    top_left = Point { x = 0, y = 0 },
    width = 100,
    height = 50,
};
```

### Доступ к полям
```rono
con.out("Координаты: ({point.x}, {point.y})");
con.out("Ширина прямоугольника: {rect.width}");
con.out("X координата верхнего левого угла: {rect.top_left.x}");
```

### Методы структур
```rono
fn_for Point {
    // Метод без возвращаемого значения
    fn print(self) {
        con.out("Point({self.x}, {self.y})");
    }
    
    // Метод с возвращаемым значением
    fn distance_from_origin(self) int {
        ret self.x * self.x + self.y * self.y;
    }
    
    // Метод, возвращающий новую структуру
    fn move_by(self, dx: int, dy: int) Point {
        var new_point: Point = Point {
            x = self.x + dx,
            y = self.y + dy,
        };
        ret new_point;
    }
}

fn_for Rectangle {
    fn area(self) int {
        ret self.width * self.height;
    }
    
    fn perimeter(self) int {
        ret 2 * (self.width + self.height);
    }
}
```

### Использование методов
```rono
chif main() {
    var point: Point = Point { x = 3, y = 4 };
    
    point.print();
    
    var distance: int = point.distance_from_origin();
    con.out("Расстояние от начала координат: {distance}");
    
    var new_point: Point = point.move_by(5, -2);
    new_point.print();
}
```

---

## 📚 Массивы и списки

### Статические массивы
```rono
// Объявление массива
var numbers: array[int] = [1, 2, 3, 4, 5];
var names: array[str] = ["Алексей", "Мария", "Иван"];

// Доступ к элементам
con.out("Первый элемент: {numbers[0]}");
con.out("Последний элемент: {numbers[4]}");

// Изменение элементов
numbers[2] = 10;
con.out("Измененный элемент: {numbers[2]}");
```

### Многомерные массивы
```rono
// Двумерный массив как массив массивов
var matrix: array[array[int]] = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
];

con.out("Элемент [1][2]: {matrix[1][2]}"); // 6
```

### Динамические списки
```rono
// Создание списка
list fruits: str[] = ["яблоко", "банан", "апельсин"];

// Получение длины
con.out("Количество фруктов: {fruits.len()}");

// Добавление элементов
fruits.add("груша");
con.out("После добавления: {fruits.len()}");

// Вставка элемента в определенную позицию
fruits.addAt("киви", 1);
con.out("После вставки: {fruits[1]}");

// Удаление элемента
fruits.del(0);
con.out("После удаления первого элемента: {fruits[0]}");
```

### Методы списков
| Метод | Описание | Пример |
|-------|----------|--------|
| `.len()` | Возвращает длину списка | `list.len()` |
| `.add(item)` | Добавляет элемент в конец | `list.add("новый")` |
| `.addAt(item, index)` | Вставляет элемент по индексу | `list.addAt("элемент", 2)` |
| `.del(index)` | Удаляет элемент по индексу | `list.del(0)` |

### Многомерные списки
```rono
// Список списков
list matrix: int[][] = [
    [1, 2, 3],
    [4, 5, 6]
];

con.out("Элемент [0][1]: {matrix[0][1]}"); // 2

// Добавление нового ряда
matrix.add([7, 8, 9]);
con.out("Новый элемент [2][0]: {matrix[2][0]}"); // 7
```

---

## 👉 Указатели и ссылки

### Создание указателей
```rono
var number: int = 42;
var number_ptr: pointer[int] = &number;

con.out("Значение: {number}");
con.out("Значение через указатель: {*number_ptr}");
```

### Указатели на структуры
```rono
struct Point {
    x: int,
    y: int,
}

chif main() {
    var point: Point = Point { x = 10, y = 20 };
    var point_ptr: pointer[Point] = &point;
    
    // Доступ к полям через указатель
    var point_ref: Point = *point_ptr;
    con.out("X через указатель: {point_ref.x}");
    con.out("Y через указатель: {point_ref.y}");
}
```

### Передача указателей в функции
```rono
fn modify_value(ptr: pointer[int], new_value: int) {
    var current: int = *ptr;
    con.out("Текущее значение: {current}");
    con.out("Новое значение: {new_value}");
    // Примечание: в текущей версии изменение через указатель
    // может потребовать дополнительной логики
}

fn print_point(ptr: pointer[Point]) {
    var point: Point = *ptr;
    con.out("Point({point.x}, {point.y})");
}

chif main() {
    var num: int = 100;
    modify_value(&num, 200);
    
    var point: Point = Point { x = 5, y = 15 };
    print_point(&point);
}
```

---

## 📦 Модули и импорты

### Создание модуля
**math_utils.rono:**
```rono
// Функции модуля
fn add(a: int, b: int) int {
    ret a + b;
}

fn multiply(a: int, b: int) int {
    ret a * b;
}

fn factorial(n: int) int {
    if (n <= 1) {
        ret 1;
    } else {
        var prev: int = factorial(n - 1);
        ret n * prev;
    }
}

// Структуры модуля
struct Calculator {
    name: str,
    version: str,
}

fn_for Calculator {
    fn info(self) {
        con.out("Калькулятор {self.name} версии {self.version}");
    }
}
```

### Использование модуля
**main.rono:**
```rono
import "math_utils";

chif main() {
    // Использование функций из модуля
    var sum: int = add(10, 20);
    con.out("Сумма: {sum}");
    
    var product: int = multiply(5, 6);
    con.out("Произведение: {product}");
    
    var fact: int = factorial(5);
    con.out("Факториал 5: {fact}");
    
    // Использование структур из модуля
    var calc: Calculator = Calculator {
        name = "RonoCalc",
        version = "1.0"
    };
    
    calc.info();
}
```

### Стандартные модули

#### string_utils.rono
```rono
struct StringProcessor {
    prefix: str,
    suffix: str,
}

fn_for StringProcessor {
    fn process_string(self, text: str) {
        con.out("{self.prefix}{text}{self.suffix}");
    }
}

fn print_greeting(name: str) {
    con.out("Привет, {name}! Добро пожаловать в Rono!");
}

fn print_banner(title: str) {
    con.out("=================================");
    con.out("         {title}");
    con.out("=================================");
}
```

---

## 🎨 Строковая интерполяция

### Базовая интерполяция
```rono
var name: str = "Алексей";
var age: int = 25;
var height: float = 175.5;
var is_student: bool = true;

con.out("Имя: {name}");
con.out("Возраст: {age} лет");
con.out("Рост: {height} см");
con.out("Студент: {is_student}");
```

### Интерполяция в одной строке
```rono
con.out("Информация: {name}, {age} лет, рост {height} см");
```

### Интерполяция со структурами
```rono
struct Person {
    name: str,
    age: int,
    city: str,
}

chif main() {
    var person: Person = Person {
        name = "Мария",
        age = 30,
        city = "Москва"
    };
    
    con.out("Имя: {person.name}");
    con.out("Возраст: {person.age}");
    con.out("Город: {person.city}");
    con.out("Полная информация: {person.name} из {person.city}, {person.age} лет");
}
```

### Интерполяция с коллекциями
```rono
list numbers: int[] = [10, 20, 30, 40, 50];

con.out("Первый элемент: {numbers[0]}");
con.out("Последний элемент: {numbers[4]}");
con.out("Длина списка: {numbers.len()}");
```

### Интерполяция с вычислениями
```rono
var a: int = 15;
var b: int = 25;

// Используем переменные для вычислений
var sum: int = a + b;
var product: int = a * b;
var average: float = (a + b) / 2.0;

con.out("Числа: a={a}, b={b}");
con.out("Сумма: {sum}");
con.out("Произведение: {product}");
con.out("Среднее: {average}");
```

---

## 🛠️ Встроенные функции

### Консольный ввод-вывод

#### Вывод в консоль
```rono
con.out("Простая строка");
con.out("Число: {42}");
con.out("Интерполяция: Hello, {'World'}!");
```

#### Ввод из консоли (если поддерживается)
```rono
// Примечание: функция con.in может быть не реализована
// в текущей версии интерпретатора
```

### Генерация случайных чисел

#### Случайные целые числа
```rono
// Случайное число от min до max (включительно)
var random_int: int = randi(1, 100);
con.out("Случайное число от 1 до 100: {random_int}");

// Несколько случайных чисел
for (i = 1; i <= 5; i = i + 1) {
    var num: int = randi(1, 10);
    con.out("Случайное число {i}: {num}");
}
```

#### Случайные числа с плавающей точкой
```rono
// Случайное число от min до max
var random_float: float = randf(0.0, 1.0);
con.out("Случайное float от 0.0 до 1.0: {random_float}");

var random_price: float = randf(10.0, 100.0);
con.out("Случайная цена: {random_price}");
```

#### Случайные строки
```rono
// Случайный символ в диапазоне
var random_char: str = rands("a", "z");
con.out("Случайная буква: {random_char}");

var random_digit: str = rands("0", "9");
con.out("Случайная цифра: {random_digit}");
```

### HTTP запросы (если поддерживается)
```rono
// Примечание: HTTP функции могут быть не полностью реализованы
// в текущей версии интерпретатора

// Примеры использования:
// var response: str = http.get("https://api.example.com/data");
// var post_result: str = http.post("https://api.example.com/submit", "data");
```

---

## 💡 Примеры программ

### 1. Hello World
```rono
chif main() {
    con.out("Hello, World!");
}
```

### 2. Калькулятор
```rono
fn add(a: int, b: int) int {
    ret a + b;
}

fn subtract(a: int, b: int) int {
    ret a - b;
}

fn multiply(a: int, b: int) int {
    ret a * b;
}

fn divide(a: int, b: int) float {
    ret a / b;
}

chif main() {
    var x: int = 20;
    var y: int = 5;
    
    con.out("Числа: {x} и {y}");
    con.out("Сложение: {add(x, y)}");
    con.out("Вычитание: {subtract(x, y)}");
    con.out("Умножение: {multiply(x, y)}");
    con.out("Деление: {divide(x, y)}");
}
```

### 3. Работа со структурами
```rono
struct Student {
    name: str,
    age: int,
    grade: float,
}

fn_for Student {
    fn print_info(self) {
        con.out("Студент: {self.name}");
        con.out("Возраст: {self.age} лет");
        con.out("Оценка: {self.grade}");
    }
    
    fn is_excellent(self) bool {
        ret self.grade >= 4.5;
    }
}

chif main() {
    var student1: Student = Student {
        name = "Алексей Иванов",
        age = 20,
        grade = 4.8,
    };
    
    var student2: Student = Student {
        name = "Мария Петрова",
        age = 19,
        grade = 4.2,
    };
    
    student1.print_info();
    if (student1.is_excellent()) {
        con.out("Отличник!");
    }
    
    con.out("---");
    
    student2.print_info();
    if (student2.is_excellent()) {
        con.out("Отличник!");
    } else {
        con.out("Хорошист");
    }
}
```

### 4. Работа со списками
```rono
chif main() {
    list numbers: int[] = [1, 2, 3, 4, 5];
    
    con.out("Исходный список:");
    for (i = 0; i < numbers.len(); i = i + 1) {
        con.out("  [{i}] = {numbers[i]}");
    }
    
    // Добавляем элементы
    numbers.add(6);
    numbers.add(7);
    numbers.addAt(0, 0); // Вставляем 0 в начало
    
    con.out("После изменений:");
    for (i = 0; i < numbers.len(); i = i + 1) {
        con.out("  [{i}] = {numbers[i]}");
    }
    
    // Удаляем элемент
    numbers.del(0); // Удаляем первый элемент
    
    con.out("После удаления первого элемента:");
    for (i = 0; i < numbers.len(); i = i + 1) {
        con.out("  [{i}] = {numbers[i]}");
    }
}
```

### 5. Факториал (рекурсия)
```rono
fn factorial(n: int) int {
    if (n <= 1) {
        ret 1;
    } else {
        var prev: int = factorial(n - 1);
        ret n * prev;
    }
}

chif main() {
    for (i = 1; i <= 10; i = i + 1) {
        var result: int = factorial(i);
        con.out("{i}! = {result}");
    }
}
```

### 6. Сортировка пузырьком
```rono
fn bubble_sort(arr: list[int]) {
    var n: int = arr.len();
    
    for (i = 0; i < n - 1; i = i + 1) {
        for (j = 0; j < n - i - 1; j = j + 1) {
            if (arr[j] > arr[j + 1]) {
                // Обмен элементов
                var temp: int = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}

chif main() {
    list numbers: int[] = [64, 34, 25, 12, 22, 11, 90];
    
    con.out("Исходный массив:");
    for (i = 0; i < numbers.len(); i = i + 1) {
        con.out("  {numbers[i]}");
    }
    
    bubble_sort(numbers);
    
    con.out("Отсортированный массив:");
    for (i = 0; i < numbers.len(); i = i + 1) {
        con.out("  {numbers[i]}");
    }
}
```

### 7. Модульная программа

**geometry.rono:**
```rono
struct Circle {
    radius: float,
}

struct Rectangle {
    width: float,
    height: float,
}

fn_for Circle {
    fn area(self) float {
        ret 3.14159 * self.radius * self.radius;
    }
    
    fn circumference(self) float {
        ret 2.0 * 3.14159 * self.radius;
    }
}

fn_for Rectangle {
    fn area(self) float {
        ret self.width * self.height;
    }
    
    fn perimeter(self) float {
        ret 2.0 * (self.width + self.height);
    }
}
```

**main.rono:**
```rono
import "geometry";

chif main() {
    var circle: Circle = Circle { radius = 5.0 };
    var rect: Rectangle = Rectangle { width = 10.0, height = 7.0 };
    
    con.out("Круг с радиусом {circle.radius}:");
    con.out("  Площадь: {circle.area()}");
    con.out("  Длина окружности: {circle.circumference()}");
    
    con.out("Прямоугольник {rect.width} x {rect.height}:");
    con.out("  Площадь: {rect.area()}");
    con.out("  Периметр: {rect.perimeter()}");
}
```

---

## ✨ Лучшие практики

### 1. Именование
```rono
// ✅ Хорошо
var user_count: int = 10;
var max_retry_attempts: int = 3;

struct UserProfile {
    first_name: str,
    last_name: str,
    email_address: str,
}

fn calculate_total_price(base_price: float, tax_rate: float) float {
    ret base_price * (1.0 + tax_rate);
}

// ❌ Плохо
var uc: int = 10;
var x: int = 3;

struct UP {
    fn: str,
    ln: str,
    ea: str,
}
```

### 2. Структура кода
```rono
// ✅ Хорошо - логическое группирование
struct Point {
    x: int,
    y: int,
}

fn_for Point {
    fn print(self) {
        con.out("Point({self.x}, {self.y})");
    }
    
    fn distance_to(self, other: Point) float {
        var dx: int = self.x - other.x;
        var dy: int = self.y - other.y;
        ret dx * dx + dy * dy; // Упрощенное расстояние
    }
}

chif main() {
    var p1: Point = Point { x = 0, y = 0 };
    var p2: Point = Point { x = 3, y = 4 };
    
    p1.print();
    p2.print();
    
    var dist: float = p1.distance_to(p2);
    con.out("Расстояние: {dist}");
}
```

### 3. Обработка ошибок
```rono
fn safe_divide(a: int, b: int) float {
    if (b == 0) {
        con.out("Ошибка: деление на ноль!");
        ret 0.0;
    } else {
        ret a / b;
    }
}

fn validate_age(age: int) bool {
    if (age < 0) {
        con.out("Ошибка: возраст не может быть отрицательным");
        ret false;
    } else {
        if (age > 150) {
            con.out("Предупреждение: возраст кажется слишком большим");
        }
        ret true;
    }
}
```

### 4. Модульность
```rono
// Разделяйте код на логические модули
// utils.rono - общие утилиты
// models.rono - структуры данных
// business_logic.rono - бизнес-логика
// main.rono - точка входа
```

### 5. Комментарии
```rono
// ✅ Хорошо - объясняют "почему", а не "что"
fn fibonacci(n: int) int {
    // Используем итеративный подход для лучшей производительности
    if (n <= 1) {
        ret n;
    }
    
    var prev: int = 0;
    var curr: int = 1;
    
    // Вычисляем числа Фибоначчи до n-го
    for (i = 2; i <= n; i = i + 1) {
        var next: int = prev + curr;
        prev = curr;
        curr = next;
    }
    
    ret curr;
}
```

---

## 🚨 Справочник по ошибкам

### Ошибки компиляции

#### 1. Неопределенная переменная
```rono
// ❌ Ошибка
chif main() {
    con.out("{undefined_var}");
}
```
**Решение:** Объявите переменную перед использованием:
```rono
// ✅ Правильно
chif main() {
    var defined_var: str = "Hello";
    con.out("{defined_var}");
}
```

#### 2. Несоответствие типов
```rono
// ❌ Ошибка
var number: int = "строка";
```
**Решение:** Используйте правильный тип:
```rono
// ✅ Правильно
var number: int = 42;
var text: str = "строка";
```

#### 3. Отсутствующая точка с запятой
```rono
// ❌ Ошибка
chif main() {
    var x: int = 10
    con.out("{x}")
}
```
**Решение:** Добавьте точки с запятой:
```rono
// ✅ Правильно
chif main() {
    var x: int = 10;
    con.out("{x}");
}
```

#### 4. Неправильное объявление функции
```rono
// ❌ Ошибка
function main() {
    con.out("Hello");
}
```
**Решение:** Используйте правильный синтаксис:
```rono
// ✅ Правильно
chif main() {
    con.out("Hello");
}
```

### Ошибки времени выполнения

#### 1. Выход за границы массива/списка
```rono
// ❌ Может вызвать ошибку
list numbers: int[] = [1, 2, 3];
con.out("{numbers[5]}"); // Индекс 5 не существует
```
**Решение:** Проверяйте границы:
```rono
// ✅ Правильно
list numbers: int[] = [1, 2, 3];
if (5 < numbers.len()) {
    con.out("{numbers[5]}");
} else {
    con.out("Индекс вне границ");
}
```

#### 2. Деление на ноль
```rono
// ❌ Может вызвать ошибку
var result: int = 10 / 0;
```
**Решение:** Проверяйте делитель:
```rono
// ✅ Правильно
var divisor: int = 0;
if (divisor != 0) {
    var result: int = 10 / divisor;
    con.out("Результат: {result}");
} else {
    con.out("Ошибка: деление на ноль");
}
```

#### 3. Неправильная работа с указателями
```rono
// ❌ Может вызвать ошибку
var ptr: pointer[int] = nil;
var value: int = *ptr; // Разыменование nil указателя
```

### Логические ошибки

#### 1. Бесконечный цикл
```rono
// ❌ Бесконечный цикл
var i: int = 0;
while (i < 10) {
    con.out("{i}");
    // Забыли увеличить i
}
```
**Решение:** Убедитесь, что условие цикла изменяется:
```rono
// ✅ Правильно
var i: int = 0;
while (i < 10) {
    con.out("{i}");
    i = i + 1; // Увеличиваем счетчик
}
```

#### 2. Неправильная логика условий
```rono
// ❌ Логическая ошибка
var age: int = 25;
if (age > 18 && age < 18) { // Невозможное условие
    con.out("Взрослый");
}
```
**Решение:** Проверьте логику:
```rono
// ✅ Правильно
var age: int = 25;
if (age >= 18) {
    con.out("Взрослый");
}
```

---

## 🎓 Заключение

Язык программирования **Rono** предоставляет мощные инструменты для создания эффективных и читаемых программ. Благодаря простому синтаксису, статической типизации и богатому набору возможностей, Rono подходит как для изучения программирования, так и для решения практических задач.

### Ключевые преимущества Rono:
- 🎯 **Простота изучения** - минимальный синтаксис
- 🔒 **Безопасность** - статическая типизация
- ⚡ **Производительность** - JIT-компиляция
- 🛠️ **Богатые возможности** - структуры, указатели, модули
- 🎨 **Выразительность** - строковая интерполяция

### Дальнейшее развитие:
- Изучите примеры в папке `examples/`
- Запустите тесты из `interpreter_test_suite/`
- Экспериментируйте с созданием собственных модулей
- Участвуйте в развитии языка на GitHub

**Добро пожаловать в мир программирования на Rono!** 🚀

---

*Документация актуальна для версии Rono 1.0.0*  
*Последнее обновление: январь 2025*