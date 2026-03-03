#!/bin/bash
set -e

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

echo ":: Загрузка gptcopy..."
# Путь к твоему будущему репозиторию (замени USERNAME на свой)
curl -sSL "https://https://github.com/Fantom1901/gptcopy/bin" -o "$INSTALL_DIR/gptcopy"
chmod +x "$INSTALL_DIR/gptcopy"

# Определение Shell и настройка PATH
SHELL_TYPE=$(basename "$SHELL")
case "$SHELL_TYPE" in
    fish) CONF="$HOME/.config/fish/config.fish";;
    zsh)  CONF="$HOME/.zshrc";;
    *)    CONF="$HOME/.bashrc";;
esac

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ":: Добавление $INSTALL_DIR в PATH..."
    if [ "$SHELL_TYPE" == "fish" ]; then
        echo "fish_add_path $INSTALL_DIR" >> "$CONF"
    else
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$CONF"
    fi
    echo ":: Перезапустите терминал или выполните 'source $CONF'"
fi

echo ":: Установка завершена. gptcopy готов к работе!"
