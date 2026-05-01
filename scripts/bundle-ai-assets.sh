#!/bin/sh
set -e

# Bundles model assets for deployment
ASSETS_DIR="dist/assets"
MODEL_FILE="model.safetensors"
TOKENIZER_FILE="tokenizer.json"

echo "Bundling AI models..."
mkdir -p "$ASSETS_DIR"

if [ -f "$MODEL_FILE" ]; then
    cp "$MODEL_FILE" "$ASSETS_DIR/"
else
    echo "Warning: $MODEL_FILE not found, skipping..."
fi

if [ -f "$TOKENIZER_FILE" ]; then
    cp "$TOKENIZER_FILE" "$ASSETS_DIR/"
else
    echo "Warning: $TOKENIZER_FILE not found, skipping..."
fi

echo "AI models bundled successfully."
