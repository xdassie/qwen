# Visual Testing Skill for Opencode

## Overview
This skill guides the Opencode agent to implement a visual testing pipeline for Bevy game development. The pipeline integrates Qwen2-VL vision model with game rendering to provide real-time visual feedback.

**Note**: All paths assume the project is in the current working directory.

---

## Component 1: Create visual_test.sh

**Purpose**: Reads base64 image from stdin, sends to Qwen2-VL vision model, outputs analysis to stdout.

```bash
#!/bin/bash

# Read base64 image from stdin
BASE64_IMAGE=$(cat)

# Check if image received
if [ -z "$BASE64_IMAGE" ]; then
    echo "Error: No base64 image received from stdin"
    exit 1
fi

# Send to Qwen2-VL vision model
VISION_PORT=${VISION_PORT:-8082}

# Build curl request with base64 image
curl -s -X POST "http://localhost:${VISION_PORT}/completions" \
    -H "Content-Type: application/json" \
    -d "{
        \"prompt\": \"<image>\\nDescribe what you see in this image. Focus ONLY on visual elements: colors, shapes, positions, orientations. Do not make any reasoning or comparisons. Report exactly what visual elements are present.\",
        \"n_predict\": 256,
        \"temperature\": 0.1,
        \"stop\": [\"\"]
    }"
```

**Make executable:**
```bash
chmod +x ./visual_test.sh
```

---

## Component 2: Modify Bevy Game to Output PNG

**File**: `/src/graphics/bevy.rs`

- Use `bevy::utils::io::write_image` to save screenshot to file
- Output image to `./screenshot.png` in current directory
- DO NOT output base64 or print to stdout/stderr

---

## Component 3: Run Test Pipeline

**Workflow sequence:**

1. **User specifies task**: "Make cube red"
2. **Agent modifies Bevy game code**
3. **Agent runs game**:
    ```bash
    cargo run --quiet
    ```
4. **Game writes screenshot to `./screenshot.png`**
5. **visual_test.sh reads PNG**, converts to base64, sends to Qwen2-VL
6. **Qwen2-VL outputs visual analysis**: "I see a red cube at position (0, 0, 5)"
7. **Analysis printed to stdout**, visible to Opencode agent

**Integration with Opencode:**
- After agent modifies code, run: `./visual_test.sh ./screenshot.png`
- visual_test.sh encodes image and calls vision model
- Opencode reads stdout output
- Agent uses visual analysis to decide next action
- Loop continues until test passes or user intervenes

---

## Component 4: Error Handling

**visual_test.sh exit codes:**
- `0`: Analysis received successfully
- `1`: Error (no image received, Qwen2-VL unreachable, curl failed)

**Error messages:**
```bash
echo "Error: Qwen2-VL vision model not running on port ${VISION_PORT}"
echo "Error: Failed to send image to vision model"
```

**Agent behavior on error:**
- Opencode should check exit code
- If error, report to user or retry after model starts

---


## Component 6: Agent Workflow Example

**Step 1: User Request**
```
User: "Make the cube rotate and be red"
```

**Step 2: Agent Modifies Code**
```rust
// Agent writes new main.rs with:
// - Rotating cube using Transform modification
// - Red color: Color::rgb(1.0, 0.0, 0.0)
// - Base64 output instead of window rendering
```

**Step 3: Agent Runs Test**
```bash
cd .
cargo run --quiet | ../visual_test.sh
```

**Step 4: Vision Model Analysis**
```
Output: "I see a red cube rotating at position (0, 0, 5). The cube is a 1.0 unit cube."
```

**Step 5: Agent Evaluates**
- Task complete? Yes (cube is red and rotating)
- If no: Modify code further
- If yes: Report success

---

## Component 7: Troubleshooting

**Issue: "Qwen2-VL not running"**
- Check if container is running: `docker ps | grep vision`
- Check if model file exists: `ls ~/models/Qwen2-VL-*.gguf`
- Verify port 8082 not in use: `netstat -tlnp | grep 8082`

**Issue: "No base64 image received"**
- Check game is outputting to stdout (not stderr)
- Verify `cargo run --quiet` suppresses cargo output
- Add debug: `echo "Base64 length: $(cat | wc -c)"` before sending

**Issue: "curl failed"**
- Check Qwen2-VL server logs: `docker logs llama-vision-server`
- Verify JSON format in curl request
- Check context size is sufficient for analysis prompt
