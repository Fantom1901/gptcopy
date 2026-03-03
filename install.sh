#!/bin/bash

# Цвета для красоты
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

INSTALL_DIR="$HOME/.local/bin"
# Определяем, где лежит папка со скриптом
SCRIPT_PATH="$(dirname "$(readlink -f "$0")")"
SCRIPT_SRC="$SCRIPT_PATH/bin/gptcopy"
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

mkdir -p "$INSTALL_DIR"

# 2. Проверяем наличие исходника. Если его нет (установка через curl), скачиваем.
if [ ! -f "$SCRIPT_SRC" ]; then
    echo -e "${BLUE}::${NC} Исходник не найден локально, скачиваем в $SCRIPT_PATH/bin/..."
    mkdir -p "$SCRIPT_PATH/bin"
    if ! curl -sSL "https://raw.githubusercontent.com/Fantom1901/gptcopy/main/bin/gptcopy" -o "$SCRIPT_SRC"; then
        echo -e "${RED}!!${NC} Ошибка: Не удалось скачать файл!"
        exit 1
    fi
fi

# 3. Создаем символьную ссылку
ln -sf "$SCRIPT_SRC" "$INSTALL_DIR/gptcopy"
chmod +x "$SCRIPT_SRC"

# 4. Добавляем в PATH если его там нет
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
echo -e "${BLUE}::${NC} Чтобы всё заработало, введите:"
echo -e "   ${GREEN}source $CONF_FILE${NC}"
echo -e "--------------------------------------------------"