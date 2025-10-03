# 🚀 Быстрое развертывание Rono Language

Пошаговая инструкция для публикации интерпретатора Rono как отдельной системы.

## 📋 Чек-лист перед публикацией

- [ ] ✅ Все тесты пройдены успешно
- [ ] 📝 Обновлены метаданные в Cargo.toml
- [ ] 📚 Создана документация (README.md, CHANGELOG.md)
- [ ] 🔧 Настроены скрипты установки
- [ ] 📦 Подготовлены файлы для пакетных менеджеров
- [ ] 🔄 Настроен GitHub Actions

## 🎯 Быстрый старт (5 минут)

### 1. Автоматическое развертывание

```bash
# Запустите скрипт автоматического развертывания
./scripts/deploy.sh
```

Этот скрипт:
- ✅ Проверит зависимости
- 🔨 Соберет проект и запустит тесты
- 📝 Поможет обновить версию
- 🏷️ Создаст Git тег
- 📋 Покажет инструкции по публикации

### 2. Создание GitHub репозитория

1. **Создайте репозиторий на GitHub:**
   - Перейдите на https://github.com/new
   - Название: `rono-lang`
   - Сделайте публичным
   - НЕ инициализируйте с README

2. **Подключите локальный репозиторий:**
   ```bash
   git remote add origin https://github.com/YOURUSERNAME/rono-lang.git
   git branch -M main
   git push -u origin main
   git push origin --tags
   ```

### 3. Создание релиза

1. Перейдите на https://github.com/YOURUSERNAME/rono-lang/releases
2. Нажмите "Create a new release"
3. Выберите тег (например, v1.0.0)
4. Заполните описание релиза
5. Опубликуйте релиз

GitHub Actions автоматически:
- 🔨 Соберет бинарные файлы для всех платформ
- 📦 Создаст архивы для скачивания
- 🚀 Опубликует релиз

## 📦 Настройка пакетных менеджеров

### Homebrew (macOS/Linux)

1. **Создайте tap репозиторий:**
   ```bash
   # Создайте репозиторий homebrew-rono на GitHub
   git clone https://github.com/YOURUSERNAME/homebrew-rono.git
   cd homebrew-rono
   mkdir Formula
   cp ../rono-lang/packaging/homebrew/rono.rb Formula/
   # Обновите SHA256 хеш в файле
   git add . && git commit -m "Add rono formula" && git push
   ```

2. **Пользователи смогут устанавливать:**
   ```bash
   brew tap YOURUSERNAME/rono
   brew install rono
   ```

### Arch Linux (AUR)

1. **Отправьте в AUR:**
   ```bash
   # Клонируйте AUR репозиторий
   git clone ssh://aur@aur.archlinux.org/rono-lang.git
   cd rono-lang
   cp ../rono-lang/packaging/arch/PKGBUILD .
   # Обновите SHA256 хеш
   makepkg --printsrcinfo > .SRCINFO
   git add . && git commit -m "Initial import" && git push
   ```

2. **Пользователи смогут устанавливать:**
   ```bash
   yay -S rono-lang
   ```

### Crates.io (Rust)

```bash
# Опубликуйте в crates.io
cargo login YOUR_API_TOKEN
cargo publish
```

## 🔧 Обновление метаданных

Перед публикацией обновите:

### Cargo.toml
```toml
[package]
name = "rono-lang"
version = "1.0.0"  # Обновите версию
authors = ["Your Name <your.email@example.com>"]  # Ваши данные
repository = "https://github.com/YOURUSERNAME/rono-lang"  # Ваш репозиторий
```

### README.md
- Замените `yourusername` на ваш GitHub username
- Обновите ссылки на репозиторий
- Добавьте актуальные примеры

### Скрипты установки
- Обновите переменную `REPO` в `scripts/install.sh` и `scripts/install.ps1`
- Замените `yourusername/rono-lang` на ваш репозиторий

## 🧪 Тестирование установки

После публикации протестируйте установку:

```bash
# macOS/Linux
curl -sSL https://raw.githubusercontent.com/YOURUSERNAME/rono-lang/main/scripts/install.sh | bash

# Windows (PowerShell)
iwr -useb https://raw.githubusercontent.com/YOURUSERNAME/rono-lang/main/scripts/install.ps1 | iex
```

## 📈 Продвижение

1. **Создайте документацию:**
   - Wiki на GitHub
   - Примеры использования
   - Туториалы

2. **Поделитесь в сообществах:**
   - Reddit (r/rust, r/ProgrammingLanguages)
   - Hacker News
   - Twitter/X
   - Dev.to

3. **Добавьте бейджи в README:**
   ```markdown
   [![Release](https://img.shields.io/github/v/release/YOURUSERNAME/rono-lang)](https://github.com/YOURUSERNAME/rono-lang/releases)
   [![Downloads](https://img.shields.io/github/downloads/YOURUSERNAME/rono-lang/total)](https://github.com/YOURUSERNAME/rono-lang/releases)
   [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
   ```

## 🎉 Готово!

После выполнения всех шагов пользователи смогут устанавливать Rono:

```bash
# Быстрая установка
curl -sSL https://raw.githubusercontent.com/YOURUSERNAME/rono-lang/main/scripts/install.sh | bash

# Homebrew
brew tap YOURUSERNAME/rono && brew install rono

# Arch Linux
yay -S rono-lang

# Cargo
cargo install rono-lang
```

**Поздравляем! Вы создали полноценный язык программирования! 🎊**