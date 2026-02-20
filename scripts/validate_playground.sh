#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY_PATH="$PROJECT_ROOT/target/debug/woven"
REPORTS_DIR="$PROJECT_ROOT/reports"
REPORT_FILE="$REPORTS_DIR/playground_report.html"

echo "🐍 WovenSnake Playground Automation (v0.2.0)"
echo "-------------------------------------------"

if [[ ! -f "$BINARY_PATH" ]]; then
    echo "Error: Binary not found at $BINARY_PATH. Run 'cargo build' first." >&2
    exit 1
fi

mkdir -p "$REPORTS_DIR"

# --- Data collectors ---
TABLE_ROWS=""
DETAILS_SECTIONS=""
STEP=1

# Run a step and capture result
run_step() {
    local name="$1"; shift
    local cmd="$1"; shift
    local args=("$@")

    printf "  Running: %s..." "$name"
    local start_ns
    start_ns=$(date +%s%N 2>/dev/null || echo 0)

    local output exit_code
    set +e
    output=$(RUST_BACKTRACE=1 RUST_LOG=debug "$cmd" "${args[@]}" 2>&1)
    exit_code=$?
    set -e

    local end_ns
    end_ns=$(date +%s%N 2>/dev/null || echo 0)
    local duration_ms=$(( (end_ns - start_ns) / 1000000 ))
    local duration_s
    duration_s=$(awk "BEGIN {printf \"%.2f\", $duration_ms / 1000}")

    if [[ $exit_code -eq 0 ]]; then
        echo " DONE (${duration_s}s)"
    else
        echo " FAILED (Exit: $exit_code) (${duration_s}s)"
        echo "    Error details captured in report."
    fi

    LAST_OUTPUT="$output"
    LAST_EXIT=$exit_code
    LAST_DURATION="${duration_s}s"
}

add_result() {
    local action="$1"
    local exit_code="$2"
    local check_passed="$3"   # "true" or "false"
    local duration="$4"
    local output="$5"

    local status_class status_text
    if [[ $exit_code -eq 0 && "$check_passed" == "true" ]]; then
        status_class="pass"; status_text="PASS"
    else
        status_class="fail"; status_text="FAIL"
    fi

    # Escape HTML special chars in output
    local clean_output
    clean_output=$(printf '%s' "$output" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g' | \
        sed 's/\x1b\[[0-9;]*m//g')

    TABLE_ROWS+="<tr><td>$STEP</td><td>$action</td><td><span class='$status_class'>$status_text</span></td><td>$duration</td></tr>"$'\n'
    DETAILS_SECTIONS+="
    <details>
        <summary>
            <span>$STEP. $action</span>
            <span class='$status_class'>$status_text</span>
        </summary>
        <div class=\"output-content\">
            <pre class=\"$([ "$status_class" = "fail" ] && echo "output-error")\">$clean_output</pre>
        </div>
    </details>"

    STEP=$(( STEP + 1 ))
}

# ── 1. Setup ────────────────────────────────────────────────────────────────
echo ""
echo "[1/10] Setting up playground..."
PLAYGROUND_DIR="$PROJECT_ROOT/playground_$$"
mkdir -p "$PLAYGROUND_DIR"
cd "$PLAYGROUND_DIR"

# ── 2. Init ─────────────────────────────────────────────────────────────────
run_step "Initialize Project (Auto-detect Python)" "$BINARY_PATH" init --yes
INIT_EXIT=$LAST_EXIT; INIT_OUTPUT="$LAST_OUTPUT"; INIT_DUR="$LAST_DURATION"

if [[ ! -f "wovenpkg.json" ]]; then
    echo "CRITICAL: wovenpkg.json was not created!" >&2
    echo "Output: $INIT_OUTPUT" >&2
    rm -rf "$PLAYGROUND_DIR"
    exit 1
fi

SYSTEM_PYTHON=$(python3 --version 2>&1 | sed 's/Python //')
SYSTEM_MAJOR_MINOR="${SYSTEM_PYTHON%%.*}.$(echo "$SYSTEM_PYTHON" | cut -d. -f2)"
CONFIG_PYTHON=$(python3 -c "import json,sys; d=json.load(open('wovenpkg.json')); print(d.get('python_version',''))")
INIT_CHECK=$([[ "$CONFIG_PYTHON" == "$SYSTEM_MAJOR_MINOR" ]] && echo "true" || echo "false")
add_result "Init (Auto-detect Python)" $INIT_EXIT "$INIT_CHECK" "$INIT_DUR" "$INIT_OUTPUT"

# ── 3. Configure Dependencies ────────────────────────────────────────────────
echo "  Configuring wovenpkg.json..."
python3 - <<'EOF'
import json
with open("wovenpkg.json") as f:
    cfg = json.load(f)
cfg["dependencies"] = {"requests": "==2.31.0"}
with open("wovenpkg.json", "w") as f:
    json.dump(cfg, f, indent=2)
EOF

# ── 4. Install ───────────────────────────────────────────────────────────────
run_step "Install Dependencies" "$BINARY_PATH" install
add_result "Install Packages" $LAST_EXIT "true" "$LAST_DURATION" "$LAST_OUTPUT"

# ── 5. Verify Lockfile Python Version ────────────────────────────────────────
if [[ -f "wovenpkg.lock" ]]; then
    LOCK_PYTHON=$(python3 -c "import json; d=json.load(open('wovenpkg.lock')); print(d.get('python_version',''))")
    LOCK_CHECK=$([[ -n "$LOCK_PYTHON" && "$LOCK_PYTHON" == "$CONFIG_PYTHON" ]] && echo "true" || echo "false")
    add_result "Verify Lockfile Python" 0 "$LOCK_CHECK" "0.00s" "Lockfile Python Version: $LOCK_PYTHON"
else
    add_result "Verify Lockfile Python" 1 "false" "0.00s" "wovenpkg.lock not found"
fi

# ── 6. Version Mismatch Warning ──────────────────────────────────────────────
echo "  Simulating Python version mismatch..."
python3 - <<'EOF'
import json
with open("wovenpkg.json") as f:
    cfg = json.load(f)
cfg["python_version"] = "3.11"
with open("wovenpkg.json", "w") as f:
    json.dump(cfg, f, indent=2)
EOF
run_step "Install with Version Mismatch" "$BINARY_PATH" install
WARN_CHECK=$([[ "$LAST_OUTPUT" == *"Existing virtual environment uses Python"* ]] && echo "true" || echo "false")
add_result "Venv Version Warning" $LAST_EXIT "$WARN_CHECK" "$LAST_DURATION" "$LAST_OUTPUT"

# ── 7. List Managed Pythons ──────────────────────────────────────────────────
run_step "List Managed Pythons" "$BINARY_PATH" list
add_result "List Managed Pythons" $LAST_EXIT "true" "$LAST_DURATION" "$LAST_OUTPUT"

# ── 8. Run Script ────────────────────────────────────────────────────────────
cat > test_app.py <<'PYEOF'
import requests
print(f'SUCCESS: Requests {requests.__version__}')
PYEOF
run_step "Run Script" "$BINARY_PATH" run python test_app.py
RUN_CHECK=$([[ "$LAST_OUTPUT" == *"SUCCESS"* ]] && echo "true" || echo "false")
add_result "Run Script" $LAST_EXIT "$RUN_CHECK" "$LAST_DURATION" "$LAST_OUTPUT"

# ── 9. Clean ─────────────────────────────────────────────────────────────────
run_step "Clean Project" "$BINARY_PATH" clean
add_result "Clean Project" $LAST_EXIT "true" "$LAST_DURATION" "$LAST_OUTPUT"

# ── 10. Clean Managed Pythons ────────────────────────────────────────────────
run_step "Clean Managed Pythons" "$BINARY_PATH" clean --python
add_result "Clean Managed Pythons" $LAST_EXIT "true" "$LAST_DURATION" "$LAST_OUTPUT"

# ── Generate HTML Report ─────────────────────────────────────────────────────
OS_INFO="$(uname -s) $(uname -m)"
DATE_NOW="$(date)"

cat > "$REPORT_FILE" <<HTMLEOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WovenSnake v0.2.0 Usability Report</title>
    <style>
        body { font-family: 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; max-width: 900px; margin: 0 auto; padding: 40px 20px; background-color: #f8f9fa; color: #212529; line-height: 1.6; }
        h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 15px; margin-bottom: 30px; letter-spacing: -0.5px; }
        h2 { color: #34495e; margin-top: 40px; }
        .summary { background: white; padding: 25px; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.05); margin-bottom: 30px; border-left: 5px solid #3498db; }
        table { width: 100%; border-collapse: separate; border-spacing: 0; margin: 25px 0; background: white; border-radius: 12px; overflow: hidden; box-shadow: 0 4px 6px rgba(0,0,0,0.05); }
        th, td { padding: 15px 20px; text-align: left; border-bottom: 1px solid #eee; }
        th { background-color: #34495e; color: white; font-weight: 600; text-transform: uppercase; font-size: 0.85em; letter-spacing: 1px; }
        tr:last-child td { border-bottom: none; }
        tr:hover td { background-color: #fdfdfd; }
        .pass { color: #27ae60; font-weight: 700; background-color: rgba(39, 174, 96, 0.1); padding: 5px 10px; border-radius: 20px; display: inline-block; font-size: 0.9em; }
        .fail { color: #e74c3c; font-weight: 700; background-color: rgba(231, 76, 60, 0.1); padding: 5px 10px; border-radius: 20px; display: inline-block; font-size: 0.9em; }
        pre { background: #2d3436; color: #dfe6e9; padding: 20px; border-radius: 8px; overflow-x: auto; font-family: 'Consolas', 'Monaco', monospace; font-size: 0.9em; white-space: pre-wrap; word-wrap: break-word; }
        .output-error { border: 2px solid #e74c3c; background: #2c1a1a; }
        details { background: white; margin-bottom: 15px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.05); border: 1px solid #eee; }
        summary { padding: 15px 20px; cursor: pointer; font-weight: 600; outline: none; list-style: none; display: flex; align-items: center; justify-content: space-between; transition: background-color 0.2s; border-radius: 8px; }
        summary:hover { background-color: #f8f9fa; }
        summary::-webkit-details-marker { display: none; }
        summary::after { content: '+'; font-size: 1.2em; color: #bdc3c7; }
        details[open] summary { border-radius: 8px 8px 0 0; border-bottom: 1px solid #eee; }
        details[open] summary::after { content: '-'; }
        .output-content { padding: 0 20px 20px; animation: fadeIn 0.3s ease-in-out; }
        @keyframes fadeIn { from { opacity: 0; transform: translateY(-10px); } to { opacity: 1; transform: translateY(0); } }
        .footer { text-align: center; margin-top: 50px; color: #95a5a6; font-size: 0.9em; }
    </style>
</head>
<body>
    <h1>WovenSnake v0.2.0 Usability Report</h1>

    <div class="summary">
        <p><strong>Date:</strong> $DATE_NOW</p>
        <p><strong>Version:</strong> v0.2.0</p>
        <p><strong>Environment:</strong> $OS_INFO (bash)</p>
        <p><strong>Features Tested:</strong> Python Auto-detection, Lockfile Versioning, Venv Validation, Managed Python Management.</p>
    </div>

    <h2>Test Execution Log</h2>
    <table>
        <thead>
            <tr><th>Step</th><th>Action</th><th>Status</th><th>Duration</th></tr>
        </thead>
        <tbody>
            $TABLE_ROWS
        </tbody>
    </table>

    <h2>Detailed Outputs</h2>
    $DETAILS_SECTIONS

    <div class="footer">Generated automatically by WovenSnake Validation Script</div>
</body>
</html>
HTMLEOF

echo ""
echo "Report generated at $REPORT_FILE"

# ── Cleanup ───────────────────────────────────────────────────────────────────
echo ""
echo "[10/10] Destroying playground..."
cd "$PROJECT_ROOT"
rm -rf "$PLAYGROUND_DIR"
echo "Playground destroyed successfully."
