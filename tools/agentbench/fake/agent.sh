#!/bin/sh
# A stand-in agent for a dry run of the harness (no model, no spend): takes the CLI's arguments,
# runs ./build and ./test once, and prints one stream-json result event costing $0.01.
# Use: AGENTBENCH_AGENT=tools/agentbench/fake/agent.sh AGENTBENCH_RESULTS=/tmp/x agentbench.py batch pilot --model fake --max-usd 0.05
cat > /dev/null
./build > /dev/null 2>&1; ./test > /dev/null 2>&1
case " $* " in *" --max-budget-usd "*) ;; *) echo "fake agent: no --max-budget-usd" >&2; exit 9;; esac
[ -n "$CLAUDE_CONFIG_DIR" ] || { echo "fake agent: no CLAUDE_CONFIG_DIR" >&2; exit 9; }
echo '{"type":"result","subtype":"success","num_turns":1,"total_cost_usd":0.01,"usage":{"input_tokens":10,"output_tokens":5}}'
