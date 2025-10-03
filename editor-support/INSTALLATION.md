# 🎨 Установка поддержки синтаксиса Rono

Инструкции по установке подсветки синтаксиса для языка Rono в различных редакторах.

## 📝 Visual Studio Code

### Автоматическая установка (рекомендуется)

1. Откройте VS Code
2. Перейдите в Extensions (Ctrl+Shift+X / Cmd+Shift+X)
3. Найдите "Rono Language Support"
4. Нажмите Install

### Ручная установка

1. Скачайте папку `vscode` из этого репозитория
2. Скопируйте её в директорию расширений VS Code:
   - **Windows**: `%USERPROFILE%\.vscode\extensions\rono-language-support`
   - **macOS**: `~/.vscode/extensions/rono-language-support`
   - **Linux**: `~/.vscode/extensions/rono-language-support`
3. Перезапустите VS Code

### Создание .vsix пакета

```bash
# Установите vsce
npm install -g @vscode/vsce

# Перейдите в папку vscode
cd editor-support/vscode

# Создайте пакет
vsce package

# Установите пакет
code --install-extension rono-language-support-1.0.0.vsix
```

## 🎯 Sublime Text

1. Откройте Sublime Text
2. Перейдите в `Preferences` → `Browse Packages`
3. Создайте папку `Rono`
4. Скопируйте файл `Rono.sublime-syntax` в эту папку
5. Перезапустите Sublime Text

Файлы `.rono` теперь будут автоматически подсвечиваться.

## 🔧 Vim/Neovim

### Ручная установка

1. Создайте директорию для синтаксиса:
   ```bash
   mkdir -p ~/.vim/syntax
   mkdir -p ~/.vim/ftdetect
   ```

2. Скопируйте файл синтаксиса:
   ```bash
   cp editor-support/vim/rono.vim ~/.vim/syntax/
   ```

3. Создайте файл определения типа файла:
   ```bash
   echo 'au BufRead,BufNewFile *.rono set filetype=rono' > ~/.vim/ftdetect/rono.vim
   ```

### Для Neovim

```bash
mkdir -p ~/.config/nvim/syntax
mkdir -p ~/.config/nvim/ftdetect
cp editor-support/vim/rono.vim ~/.config/nvim/syntax/
echo 'au BufRead,BufNewFile *.rono set filetype=rono' > ~/.config/nvim/ftdetect/rono.vim
```

### С помощью пакетного менеджера

#### vim-plug
Добавьте в `.vimrc`:
```vim
Plug 'EvgeniiAndronov/Rono', {'rtp': 'editor-support/vim'}
```

#### Vundle
```vim
Plugin 'EvgeniiAndronov/Rono'
```

## 🌟 Emacs

1. Скачайте файл `rono-mode.el` (будет создан позже)
2. Поместите его в директорию `~/.emacs.d/lisp/`
3. Добавьте в `.emacs`:
   ```elisp
   (add-to-list 'load-path "~/.emacs.d/lisp/")
   (require 'rono-mode)
   (add-to-list 'auto-mode-alist '("\\.rono\\'" . rono-mode))
   ```

## ⚛️ Atom

1. Откройте Atom
2. Перейдите в `File` → `Settings` → `Install`
3. Найдите "language-rono" (если опубликован)
4. Или установите вручную:
   ```bash
   cd ~/.atom/packages
   git clone https://github.com/EvgeniiAndronov/Rono.git language-rono
   cd language-rono
   apm link
   ```

## 🔍 JetBrains IDEs (IntelliJ IDEA, CLion, etc.)

1. Откройте IDE
2. Перейдите в `File` → `Settings` → `Plugins`
3. Найдите "Rono Language Support" (если опубликован)
4. Или создайте custom file type:
   - `File` → `Settings` → `Editor` → `File Types`
   - Нажмите `+` и создайте новый тип файла
   - Добавьте расширение `*.rono`
   - Настройте подсветку синтаксиса

## 🌐 Monaco Editor (VS Code в браузере)

Для веб-редакторов на основе Monaco Editor:

```javascript
// Регистрация языка
monaco.languages.register({ id: 'rono' });

// Настройка токенизации
monaco.languages.setMonarchTokensProvider('rono', {
  tokenizer: {
    root: [
      [/\b(chif|fn|struct|var|list|array|fn_for)\b/, 'keyword'],
      [/\b(if|else|for|while|break|continue|ret|import)\b/, 'keyword.control'],
      [/\b(int|float|bool|str|pointer)\b/, 'type'],
      [/\b(true|false|nil)\b/, 'constant'],
      [/\b\d+(\.\d+)?\b/, 'number'],
      [/"([^"\\]|\\.)*"/, 'string'],
      [/\/\/.*$/, 'comment'],
      [/\/\*[\s\S]*?\*\//, 'comment']
    ]
  }
});

// Настройка автодополнения
monaco.languages.setLanguageConfiguration('rono', {
  comments: {
    lineComment: '//',
    blockComment: ['/*', '*/']
  },
  brackets: [
    ['{', '}'],
    ['[', ']'],
    ['(', ')']
  ],
  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' }
  ]
});
```

## 🧪 Тестирование

После установки создайте тестовый файл `test.rono`:

```rono
// Тест подсветки синтаксиса
import "math_utils";

struct Point {
    x: int,
    y: float,
}

fn_for Point {
    fn distance(self) float {
        ret sqrt(self.x * self.x + self.y * self.y);
    }
}

chif main() {
    var point: Point = Point { x = 3, y = 4.0 };
    list numbers: int[] = [1, 2, 3, 4, 5];
    
    // Add element to list
    numbers.add(6);
    
    con.out("Distance: {point.distance()}");
    con.out("List length: {numbers.len()}");
    
    // Random number generation
    var random_num: int = randi(1, 100);
    con.out("Random number: {random_num}");
    
    // Loop through list
    for (i = 0; i < numbers.len(); i = i + 1) {
        if (numbers[i] > 2) {
            con.out("Number: {numbers[i]}");
        }
    }
    
    // HTTP request
    var response: str = http.get("https://api.example.com");
    if (response != nil) {
        con.out("Response received");
    }
}
```

Если подсветка работает корректно, вы должны увидеть:
- 🔵 Ключевые слова выделены синим
- 🟢 Строки выделены зеленым
- 🟡 Числа выделены желтым
- 🔴 Комментарии выделены серым/красным
- 🟣 Типы данных выделены фиолетовым

## 📚 Дополнительные возможности

### Сниппеты в VS Code

После установки расширения доступны сниппеты:
- `main` → создать главную функцию
- `fn` → создать функцию
- `struct` → создать структуру
- `if` → условный оператор
- `for` → цикл for
- `cout` → вывод в консоль

### Автодополнение

В VS Code доступно базовое автодополнение для:
- Ключевых слов языка
- Встроенных функций
- Типов данных

## 🆘 Поддержка

Если у вас возникли проблемы с установкой:

1. Проверьте, что файлы скопированы в правильные директории
2. Перезапустите редактор
3. Создайте issue в [репозитории](https://github.com/EvgeniiAndronov/Rono/issues)

## 🎉 Готово!

Теперь вы можете комфортно программировать на языке Rono с полной поддержкой синтаксиса! 🚀