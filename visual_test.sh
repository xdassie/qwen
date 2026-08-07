#!/bin/bash

DEBUG=false
if [ "${DEBUG:-false}" = "true" ]; then
    DEBUG=true
fi

# Accept optional output mp4 file argument, default to output.mp4
OUTPUT_FILE="${1:-output.mp4}"

# Find all screenshots in /tmp
SCREENSHOTS=$(ls -t /tmp/screenshot_*.png 2>/dev/null)

if [ -z "$SCREENSHOTS" ]; then
    echo "Error: No screenshots found in /tmp"
    exit 1
fi

# Count screenshots
SCREENSHOT_COUNT=$(echo "$SCREENSHOTS" | wc -w)
echo "Found $SCREENSHOT_COUNT screenshots in /tmp"

# Create temp directory for processing
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Copy screenshots to temp dir with sequential names
i=1
for screenshot in $SCREENSHOTS; do
    cp "$screenshot" "$TEMP_DIR/frame_$(printf '%05d' $i).png"
    i=$((i + 1))
done

# Create mp4 using ffmpeg
echo "Creating MP4 from $SCREENSHOT_COUNT screenshots..."
ffmpeg -framerate 10 -i "$TEMP_DIR"/frame_%05d.png -vf "fps=10,scale=800:600" -c:v libx264 -preset fast -crf 28 "$OUTPUT_FILE" -y 2>&1

if [ $? -ne 0 ]; then
    echo "Error: Failed to create MP4"
    exit 1
fi

echo "MP4 created: $OUTPUT_FILE"

# Convert to base64
BASE64_FILE=$(mktemp)
base64 -w0 "$OUTPUT_FILE" > "$BASE64_FILE"
BASE64_VIDEO=$(cat "$BASE64_FILE")
rm "$BASE64_FILE"

if [ -z "$BASE64_VIDEO" ]; then
    echo "Error: Failed to convert MP4 to base64"
    exit 1
fi

if [ "$DEBUG" = "true" ]; then
    echo "Base64 length: $(echo -n "$BASE64_VIDEO" | wc -c)" >&2
fi

VISION_PORT=${VISION_PORT:-8082}

# Escape special characters in base64 for JSON
BASE64_ESCAPED=$(echo -n "$BASE64_VIDEO" | sed 's/\\/\\\\/g; s/"/\\\"/g; s/\\t/\\t/g; s/\\r/\\r/g; s/\\n/\\n/g')

# OpenAI-compatible format with video_url
PROMPT_JSON='{"model": "Qwen2-VL-7B-Instruct-Q5_K_S.gguf", "messages": [{"role": "user", "content": [{"type": "video_url", "video_url": {"url": "data:video/mp4;base64,'$BASE64_ESCAPED'"}}, {"type": "text", "text": "Analyze this video of the 3D model. Focus ONLY on visual elements: colors, shapes, positions, orientations, camera movements. Do not make any reasoning or comparisons. Report exactly what visual elements are present."}]}], "max_tokens": 512, "temperature": 0.1}'

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
