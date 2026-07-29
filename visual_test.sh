#!/bin/bash

DEBUG=false
if [ "${DEBUG:-false}" = "true" ]; then
    DEBUG=true
fi

# Accept optional image file argument, default to screenshot.png
IMAGE_FILE="${1:-screenshot.png}"

if [ ! -f "$IMAGE_FILE" ]; then
    echo "Error: $IMAGE_FILE not found"
    exit 1
fi

# Create temp file for base64
BASE64_FILE=$(mktemp)
base64 -w0 "$IMAGE_FILE" > "$BASE64_FILE"
BASE64_IMAGE=$(cat "$BASE64_FILE")
rm "$BASE64_FILE"

if [ -z "$BASE64_IMAGE" ]; then
    echo "Error: Failed to read $IMAGE_FILE"
    exit 1
fi

if [ "$DEBUG" = "true" ]; then
    echo "Base64 length: $(echo -n "$BASE64_IMAGE" | wc -c)" >&2
fi

VISION_PORT=${VISION_PORT:-8082}

# Escape special characters in base64 for JSON
BASE64_ESCAPED=$(echo -n "$BASE64_IMAGE" | sed 's/\\/\\\\/g; s/"/\\\"/g; s/\\t/\\t/g; s/\\r/\\r/g; s/\\n/\\n/g')

# OpenAI-compatible format with image_url
PROMPT_JSON='{"model": "Qwen2-VL-7B-Instruct-Q5_K_S.gguf", "messages": [{"role": "user", "content": [{"type": "image_url", "image_url": {"url": "data:image/png;base64,'$BASE64_ESCAPED'"}}, {"type": "text", "text": "Describe this image. Focus ONLY on visual elements: colors, shapes, positions, orientations. Do not make any reasoning or comparisons. Report exactly what visual elements are present."}]}], "max_tokens": 512, "temperature": 0.1}'

echo "Sending vision request to port ${VISION_PORT}..."
TMP_JSON=$(mktemp)
echo "$PROMPT_JSON" > "$TMP_JSON"
RESPONSE=$(curl -s --max-time 60 -X POST "http://localhost:${VISION_PORT}/v1/chat/completions"     -H "Content-Type: application/json"     -d @"$TMP_JSON")
rm "$TMP_JSON"

if [ $? -ne 0 ]; then
    echo "Error: Vision model not responding on port ${VISION_PORT}"
    exit 1
fi

if [ -z "$RESPONSE" ]; then
    echo "Error: Failed to receive response from vision model"
    exit 1
fi

# Extract just the content text from the response
VISUAL_ANALYSIS=$(echo "$RESPONSE" | grep -oP '(?<="content":")[^"]*' | head -1)
if [ -z "$VISUAL_ANALYSIS" ]; then
    echo "Error: Could not extract analysis from response"
    echo "Response: $RESPONSE"
    exit 1
fi

echo "$VISUAL_ANALYSIS"
