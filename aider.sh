MAIN_MODEL="openai/qwen3.5-coder-9b-instruct-q5_k_m.gguf" 
#EDITOR_MODEL="openai/qwen2.5-coder-7b-instruct-q4_k_m.gguf"
API_BASE="http://localhost:8081/v1"
~/venv_aider/bin/aider --max-chat-history-tokens 50000 --timeout 9600 --stream  --llm-history-file /tmp/llm.log --model "$MAIN_MODEL" --openai-api-base "$API_BASE"   --openai-api-key "local-key"  --git   --map-tokens 0  --no-show-model-warnings --test-cmd='cargo check'  --no-auto-commits --notifications --notifications-command="ffplay -nodisp -autoexit ~/Documents/helpme.mp3" "$@"
#cargo check 2>&1 | head -10; exit ${PIPESTATUS[0]}
