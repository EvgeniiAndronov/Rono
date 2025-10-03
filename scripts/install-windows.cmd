@echo off
setlocal enabledelayedexpansion

echo 🚀 Установка интерпретатора языка Rono
echo ==================================================

REM Проверка PowerShell
powershell -Command "Get-Host" >nul 2>&1
if errorlevel 1 (
    echo ❌ PowerShell не найден
    pause
    exit /b 1
)

REM Проверка git
git --version >nul 2>&1
if errorlevel 1 (
    echo ❌ Git не найден. Установите Git для Windows: https://git-scm.com/download/win
    pause
    exit /b 1
)

REM Проверка Rust/Cargo
cargo --version >nul 2>&1
if errorlevel 1 (
    echo 📦 Установка Rust...
    powershell -Command "Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile '%TEMP%\rustup-init.exe'"
    %TEMP%\rustup-init.exe -y
    call "%USERPROFILE%\.cargo\env.cmd"
    del "%TEMP%\rustup-init.exe"
)

echo ✅ Все зависимости найдены

REM Создание временной директории
set "TEMP_DIR=%TEMP%\rono-install-%RANDOM%"
mkdir "%TEMP_DIR%"
cd /d "%TEMP_DIR%"

echo 📥 Клонирование репозитория...
git clone https://github.com/EvgeniiAndronov/Rono.git
cd Rono

echo 🔨 Сборка проекта (это может занять несколько минут)...
cargo build --release

REM Создание директории установки
set "INSTALL_DIR=%LOCALAPPDATA%\Programs\Rono"
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

REM Копирование бинарного файла
copy "target\release\rono.exe" "%INSTALL_DIR%\"

echo 🔧 Добавление в PATH...
REM Добавление в PATH пользователя
for /f "tokens=2*" %%A in ('reg query "HKCU\Environment" /v PATH 2^>nul') do set "CURRENT_PATH=%%B"
if not defined CURRENT_PATH set "CURRENT_PATH="

echo !CURRENT_PATH! | findstr /C:"%INSTALL_DIR%" >nul
if errorlevel 1 (
    if defined CURRENT_PATH (
        reg add "HKCU\Environment" /v PATH /t REG_EXPAND_SZ /d "!CURRENT_PATH!;%INSTALL_DIR%" /f
    ) else (
        reg add "HKCU\Environment" /v PATH /t REG_EXPAND_SZ /d "%INSTALL_DIR%" /f
    )
    echo ✅ Добавлено в PATH
) else (
    echo ⚠️  Директория уже в PATH
)

REM Очистка
cd /d "%TEMP%"
rmdir /s /q "%TEMP_DIR%"

echo 🧪 Проверка установки...
if exist "%INSTALL_DIR%\rono.exe" (
    echo ✅ rono.exe успешно установлен в %INSTALL_DIR%
    echo.
    echo 🎉 Готово! Перезапустите командную строку и используйте команду 'rono'
    echo 💡 Попробуйте: rono --help
    echo 📚 Документация: https://github.com/EvgeniiAndronov/Rono
) else (
    echo ❌ Установка не удалась
    pause
    exit /b 1
)

pause