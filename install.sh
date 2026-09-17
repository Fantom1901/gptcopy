#!/bin/bash

# Цвета для красоты
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

REPO="Fantom1901/gptcopy"
BASE_DIR="$HOME/scripts/gptcopy"
INSTALL_DIR="$HOME/.local/bin"
SCRIPT_SRC="$BASE_DIR/bin/gptcopy"
SHELL_NAME=$(basename "$SHELL")

# Определяем файл конфига
case "$SHELL_NAME" in
    fish) CONF_FILE="$HOME/.config/fish/config.fish" ;;
    zsh)  CONF_FILE="$HOME/.zshrc" ;;
    *)    CONF_FILE="$HOME/.bashrc" ;;
esac

# 5. Генерация автодополнений для шелла
echo -e "${BLUE}::${NC} Генерация автодополнений для $SHELL_NAME..."
case "$SHELL_NAME" in
    fish)
        COMP_DIR="$HOME/.config/fish/completions"
        mkdir -p "$COMP_DIR"
        "$INSTALL_DIR/gptcopy" completions fish > "$COMP_DIR/gptcopy.fish"
        ;;
    zsh)
        COMP_DIR="$HOME/.zsh/completion"
        mkdir -p "$COMP_DIR"
        "$INSTALL_DIR/gptcopy" completions zsh > "$COMP_DIR/_gptcopy"
        ;;
    bash)
        COMP_DIR="$HOME/.local/share/bash-completion/completions"
        mkdir -p "$COMP_DIR"
        "$INSTALL_DIR/gptcopy" completions bash > "$COMP_DIR/gptcopy"
        ;;
esac

uninstall() {
    echo -e "${BLUE}::${NC} Удаление gptcopy..."
    rm -f "$INSTALL_DIR/gptcopy"

    # Удаление автодополнений
      rm -f "$HOME/.config/fish/completions/gptcopy.fish"
      rm -f "$HOME/.zsh/completion/_gptcopy"
      rm -f "$HOME/.local/share/bash-completion/completions/gptcopy"

    if [ -f "$CONF_FILE" ]; then
        sed -i '/gptcopy/d' "$CONF_FILE"
        if [ "$SHELL_NAME" == "fish" ]; then
            sed -i '/fish_add_path.*\.local\/bin/d' "$CONF_FILE"
        fi
    fi
    echo -e "${GREEN}::${NC} Утилита удалена. Перезапустите терминал."
    exit 0
}

if [[ "$1" == "--uninstall" ]]; then
    uninstall
fi

echo -e "${BLUE}::${NC} Установка gptcopy из GitHub Releases..."

# 1. Создаем структуру папок
mkdir -p "$INSTALL_DIR"
mkdir -p "$BASE_DIR/bin"

# 2. Если бинарника/скрипта нет локально — получаем свежий релиз
if [ ! -f "$SCRIPT_SRC" ]; then
    echo -e "${BLUE}::${NC} Поиск последнего релиза..."

    # Получаем URL скачивания прямо из GitHub API
    RELEASE_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "browser_download_url" | cut -d '"' -f 4)

    # Запасной вариант (fallback): если ассетов в релизе нет, забираем исходник из релиза по тегу
    if [ -z "$RELEASE_URL" ]; then
        echo -e "${BLUE}::${NC} Ассеты релиза не найдены, запрашиваем последнюю версию (tag)..."
        LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

        if [ -n "$LATEST_TAG" ]; then
            RELEASE_URL="https://raw.githubusercontent.com/$REPO/$LATEST_TAG/bin/gptcopy"
        else
            echo -e "${RED}!!${NC} Ошибка: Не удалось получить данные о последнем релизе!"
            exit 1
        fi
    fi

    echo -e "${BLUE}::${NC} Загрузка с $RELEASE_URL..."
    if ! curl -sSL "$RELEASE_URL" -o "$SCRIPT_SRC"; then
        echo -e "${RED}!!${NC} Ошибка: Не удалось скачать gptcopy!"
        exit 1
    fi
fi

# 3. Настраиваем исполняемый файл и симлинк
chmod +x "$SCRIPT_SRC"
ln -sf "$SCRIPT_SRC" "$INSTALL_DIR/gptcopy"

# 4. Добавляем в PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${BLUE}::${NC} Добавление в PATH ($SHELL_NAME)..."
    if [ "$SHELL_NAME" == "fish" ]; then
        echo "fish_add_path $INSTALL_DIR" >> "$CONF_FILE"
    else
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$CONF_FILE"
    fi
fi

echo -e "--------------------------------------------------"
echo -e "${GREEN}:: Установка завершена успешно!${NC}"
echo -e "${BLUE}::${NC} Локация: $SCRIPT_SRC"
echo -e "${BLUE}::${NC} Чтобы изменения вступили в силу, введите:"
echo -e "   ${GREEN}source $CONF_FILE${NC}"
echo -e "--------------------------------------------------"