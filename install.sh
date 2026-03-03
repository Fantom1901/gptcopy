#!/bin/bash

# Цвета для красоты
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ПУТИ - теперь жестко привязаны к твоей папке скриптов
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

uninstall() {
    echo -e "${BLUE}::${NC} Удаление gptcopy..."
    rm -f "$INSTALL_DIR/gptcopy"

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

echo -e "${BLUE}::${NC} Установка gptcopy..."

# 1. Создаем структуру папок
mkdir -p "$INSTALL_DIR"
mkdir -p "$BASE_DIR/bin"

# 2. Проверяем наличие исходника. Если его нет (установка через curl), скачиваем в базу.
if [ ! -f "$SCRIPT_SRC" ]; then
    echo -e "${BLUE}::${NC} Загрузка исполняемого файла в $SCRIPT_SRC..."
    if ! curl -sSL "https://raw.githubusercontent.com/Fantom1901/gptcopy/main/bin/gptcopy" -o "$SCRIPT_SRC"; then
        echo -e "${RED}!!${NC} Ошибка: Не удалось скачать gptcopy!"
        exit 1
    fi
fi

# 3. Создаем символьную ссылку (теперь путь всегда верный)
ln -sf "$SCRIPT_SRC" "$INSTALL_DIR/gptcopy"
chmod +x "$SCRIPT_SRC"

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
echo -e "${BLUE}::${NC} Чтобы всё заработало, введите:"
echo -e "   ${GREEN}source $CONF_FILE${NC}"
echo -e "--------------------------------------------------"