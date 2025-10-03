# Rono Language Installer for Windows
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\Rono",
    [switch]$AddToPath = $true,
    [string]$Repo = "EvgeniiAndronov/Rono"
)

$ErrorActionPreference = "Stop"

# Цвета для вывода
function Write-ColorOutput($ForegroundColor, $Message) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    Write-Output $Message
    $host.UI.RawUI.ForegroundColor = $fc
}

Write-ColorOutput Blue "🚀 Установка интерпретатора языка Rono"
Write-Output "=================================================="

$BINARY_NAME = "rono.exe"

# Проверка PowerShell версии
if ($PSVersionTable.PSVersion.Major -lt 5) {
    Write-ColorOutput Red "❌ Требуется PowerShell 5.0 или выше"
    exit 1
}

# Проверка зависимостей
Write-ColorOutput Yellow "📋 Проверка зависимостей..."

# Проверка git
if (!(Get-Command git -ErrorAction SilentlyContinue)) {
    Write-ColorOutput Red "❌ Git не найден. Установите Git для Windows: https://git-scm.com/download/win"
    exit 1
}

# Проверка Rust/Cargo
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-ColorOutput Yellow "📦 Установка Rust..."
    try {
        # Загрузка и установка Rust
        $RustInstaller = "$env:TEMP\rustup-init.exe"
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $RustInstaller
        Start-Process -FilePath $RustInstaller -ArgumentList "-y" -Wait
        
        # Обновление PATH для текущей сессии
        $env:PATH += ";$env:USERPROFILE\.cargo\bin"
        
        Remove-Item $RustInstaller
        Write-ColorOutput Green "✅ Rust установлен"
    } catch {
        Write-ColorOutput Red "❌ Ошибка установки Rust: $_"
        exit 1
    }
}

Write-ColorOutput Green "✅ Все зависимости найдены"

# Создание директории установки
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Write-ColorOutput Green "📁 Создана директория: $InstallDir"
}

# Клонирование и сборка из исходного кода
Write-ColorOutput Yellow "🔨 Установка из исходного кода..."

$TempDir = "$env:TEMP\rono-build-$(Get-Random)"
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

try {
    # Клонирование репозитория
    Write-ColorOutput Yellow "📥 Клонирование репозитория..."
    Set-Location $TempDir
    git clone "https://github.com/$Repo.git"
    Set-Location "Rono"
    
    # Сборка проекта
    Write-ColorOutput Yellow "🔨 Сборка проекта (это может занять несколько минут)..."
    cargo build --release
    
    # Копирование бинарного файла
    $SourceBinary = "target\release\rono.exe"
    $TargetBinary = Join-Path $InstallDir $BINARY_NAME
    
    if (Test-Path $SourceBinary) {
        Copy-Item $SourceBinary $TargetBinary -Force
        Write-ColorOutput Green "✅ Бинарный файл скопирован"
    } else {
        Write-ColorOutput Red "❌ Бинарный файл не найден после сборки"
        exit 1
    }
    
} catch {
    Write-ColorOutput Red "❌ Ошибка сборки: $_"
    exit 1
} finally {
    # Очистка временных файлов
    Set-Location $env:TEMP
    if (Test-Path $TempDir) {
        Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# Добавление в PATH
if ($AddToPath) {
    Write-ColorOutput Yellow "🔧 Добавление в PATH..."
    
    # Получение текущего PATH пользователя
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    
    if ($CurrentPath -notlike "*$InstallDir*") {
        $NewPath = if ($CurrentPath) { "$CurrentPath;$InstallDir" } else { $InstallDir }
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
        
        # Обновление PATH для текущей сессии
        $env:PATH += ";$InstallDir"
        
        Write-ColorOutput Green "✅ Добавлено в PATH"
    } else {
        Write-ColorOutput Yellow "⚠️  Директория уже в PATH"
    }
}

# Проверка установки
Write-ColorOutput Yellow "🧪 Проверка установки..."
$RonoPath = Join-Path $InstallDir $BINARY_NAME

if (Test-Path $RonoPath) {
    Write-ColorOutput Green "✅ $BINARY_NAME успешно установлен в $InstallDir"
    
    # Попытка получить версию
    try {
        $Version = & $RonoPath --version 2>$null
        if ($Version) {
            Write-ColorOutput Green "📋 Версия: $Version"
        }
    } catch {
        Write-ColorOutput Yellow "⚠️  Не удалось получить версию, но файл установлен"
    }
    
    Write-Output ""
    Write-ColorOutput Blue "🎉 Готово! Перезапустите терминал и используйте команду 'rono'"
    Write-ColorOutput Blue "💡 Попробуйте: rono --help"
    Write-ColorOutput Blue "📚 Документация: https://github.com/$Repo"
    
    if (!$AddToPath) {
        Write-ColorOutput Yellow "⚠️  Не забудьте добавить $InstallDir в PATH вручную"
    }
} else {
    Write-ColorOutput Red "❌ Установка не удалась"
    exit 1
}