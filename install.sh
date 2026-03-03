#!/bin/bash
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
echo ":: Загрузка gptcopy..."
curl -sSL https://raw.githubusercontent.com/Fantom1901/gptcopy/main/bin/gptcopy -o "$INSTALL_DIR/gptcopy"
chmod +x "$INSTALL_DIR/gptcopy"
echo ":: Установка завершена!"
