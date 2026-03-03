#!/bin/bash
set -e

# 1. Настройка путей
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

echo ":: Загрузка gptcopy..."
# Скачиваем основной исполняемый файл
curl -sSL "https://raw.githubusercontent.com/Fantom1901/gptcopy/main/bin/gptcopy" -o "$INSTALL_DIR/gptcopy"
chmod +x "$INSTALL_DIR/gptcopy"

# 2. Определение текущего Shell и настройка PATH
SHELL_NAME=$(basename "$SHELL")
case "$SHELL_NAME" in
    fish)
        CONF_FILE="$HOME/.config/fish/config.fish"
        ADD_COMMAND="fish_add_path $INSTALL_DIR"
        ;;
    zsh)
        CONF_FILE="$HOME/.zshrc"
        ADD_COMMAND="export PATH=\"\$PATH:$INSTALL_DIR\""
        ;;
    *)
        CONF_FILE="$HOME/.bashrc"
        ADD_COMMAND="export PATH=\"\$PATH:$INSTALL_DIR\""
        ;;
esac

# Проверяем, есть ли уже путь в PATH, чтобы не дублировать
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ":: Добавление $INSTALL_DIR в PATH ($SHELL_NAME)..."
    echo "$ADD_COMMAND" >> "$CONF_FILE"
    echo ":: Путь добавлен в $CONF_FILE"
fi

echo "--------------------------------------------------"
echo ":: Установка завершена успешно!"
echo ":: Чтобы изменения вступили в силу, введите:"
echo "   source $CONF_FILE"
echo "--------------------------------------------------"
