#!/bin/bash
# Универсальный инсталлятор gptcopy

# 1. Определяем папку установки
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# 2. Качаем сам скрипт
echo ":: Загрузка gptcopy..."
curl -sSL "https://raw.githubusercontent.com/Fantom1901/gptcopy/main/bin/gptcopy" -o "$INSTALL_DIR/gptcopy"

# 3. Делаем исполняемым
chmod +x "$INSTALL_DIR/gptcopy"

# 4. Добавляем в PATH если нужно
SHELL_NAME=$(basename "$SHELL")
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ":: Добавление в PATH для $SHELL_NAME..."
    case "$SHELL_NAME" in
        fish) echo "fish_add_path $INSTALL_DIR" >> "$HOME/.config/fish/config.fish" ;;
        zsh)  echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$HOME/.zshrc" ;;
        *)    echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$HOME/.bashrc" ;;
    esac
fi

echo ":: Установка завершена! Перезапустите терминал."
