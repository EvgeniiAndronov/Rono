# 🔧 Решение проблем установки Rono

## 🍎 macOS: Ошибка "Is a directory"

**Проблема:** `cp: /var/folders/.../rono: Is a directory`

**Решение:**
```bash
# Вариант 1: Ручная установка
git clone https://github.com/EvgeniiAndronov/Rono.git
cd Rono
cargo build --release
sudo cp target/release/rono /usr/local/bin/

# Вариант 2: Проверьте PATH
echo $PATH | grep /usr/local/bin
# Если /usr/local/bin не в PATH, добавьте:
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

## 🪟 Windows: Проблемы с curl

**Проблема:** `curl: option -sSL: is ambiguous` или `curl не найден`

**Решение 1 - Альтернативный скрипт:**
```cmd
curl -L https://raw.githubusercontent.com/EvgeniiAndronov/Rono/main/scripts/install-windows.cmd -o install.cmd
install.cmd
```

**Решение 2 - PowerShell без curl:**
```powershell
# Скачайте и запустите PowerShell скрипт напрямую
$script = Invoke-WebRequest -Uri "https://raw.githubusercontent.com/EvgeniiAndronov/Rono/main/scripts/install.ps1" -UseBasicParsing
Invoke-Expression $script.Content
```

**Решение 3 - Ручная установка:**
```cmd
# 1. Установите Git: https://git-scm.com/download/win
# 2. Установите Rust: https://rustup.rs/
# 3. Клонируйте и соберите:
git clone https://github.com/EvgeniiAndronov/Rono.git
cd Rono
cargo build --release
# 4. Скопируйте rono.exe в папку в PATH или добавьте папку target/release в PATH
```

## 🐧 Linux: Проблемы с правами

**Проблема:** `Permission denied` при копировании в `/usr/local/bin`

**Решение:**
```bash
# Вариант 1: Использовать sudo
sudo cp target/release/rono /usr/local/bin/

# Вариант 2: Установить в домашнюю папку
mkdir -p ~/.local/bin
cp target/release/rono ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## 🔍 Проверка установки

После установки проверьте:

```bash
# Проверьте, что rono в PATH
which rono

# Проверьте версию
rono --version

# Запустите простой тест
echo 'chif main() { con.out("Hello, Rono!"); }' > test.rono
rono run test.rono
rm test.rono
```

## 🆘 Если ничего не помогает

1. **Проверьте зависимости:**
   - Git установлен и работает
   - Rust/Cargo установлен и работает
   - Интернет соединение стабильно

2. **Ручная установка:**
   ```bash
   git clone https://github.com/EvgeniiAndronov/Rono.git
   cd Rono
   cargo build --release
   # Скопируйте target/release/rono в любую папку в PATH
   ```

3. **Создайте issue:**
   - Перейдите на https://github.com/EvgeniiAndronov/Rono/issues
   - Опишите проблему с указанием ОС и версии
   - Приложите вывод команд `uname -a` и `cargo --version`

## 📞 Поддержка

- 🐛 [Сообщить об ошибке](https://github.com/EvgeniiAndronov/Rono/issues)
- 💬 [Обсуждения](https://github.com/EvgeniiAndronov/Rono/discussions)
- 📚 [Документация](https://github.com/EvgeniiAndronov/Rono)