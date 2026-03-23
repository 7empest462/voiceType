#!/bin/bash
# AI Helper Suite - Unified Launcher
# Usage: ./ai_helpers.sh [command]

SCRIPTS_DIR="/Volumes/Corsair_Lab/scripts"
PYTHON="$SCRIPTS_DIR/venv/bin/python"

show_help() {
    echo ""
    echo "🤖 AI Helper Suite"
    echo "=================="
    echo ""
    echo "Commands:"
    echo "  monitor    - System performance monitor"
    echo "  organize   - Smart file organizer (dry run)"
    echo "  organize!  - Smart file organizer (execute)"
    echo "  guardian   - Disk space guardian"
    echo "  logs       - Log analyzer"
    echo "  boost      - Performance mode"
    echo "  voice      - Voice to text (Typeless replacement)"
    echo "  ask        - AI Code Consultant (reads stdin)"
    echo "  all        - Run all checks"
    echo ""
    echo "Examples:"
    echo "  ./ai_helpers.sh monitor"
    echo "  ./ai_helpers.sh organize"
    echo "  ./ai_helpers.sh all"
    echo ""
}

case "$1" in
    monitor)
        $PYTHON "$SCRIPTS_DIR/system_monitor.py" "${@:2}"
        ;;
    organize)
        $PYTHON "$SCRIPTS_DIR/file_organizer.py" "${@:2}"
        ;;
    organize!)
        $PYTHON "$SCRIPTS_DIR/file_organizer.py" -e "${@:2}"
        ;;
    guardian)
        $PYTHON "$SCRIPTS_DIR/disk_guardian.py" "${@:2}"
        ;;
    logs)
        $PYTHON "$SCRIPTS_DIR/log_analyzer.py" "${@:2}"
        ;;
    boost)
        $PYTHON "$SCRIPTS_DIR/performance_mode.py" "${@:2}"
        ;;
    voice)
        $PYTHON "$SCRIPTS_DIR/voice_type.py" "${@:2}"
        ;;
    ask)
        $PYTHON "$SCRIPTS_DIR/ai_consultant.py" "${@:2}"
        ;;
    all)
        echo "🤖 Running all AI helpers..."
        echo ""
        $PYTHON "$SCRIPTS_DIR/system_monitor.py" --no-ai
        $PYTHON "$SCRIPTS_DIR/disk_guardian.py" --no-ai
        $PYTHON "$SCRIPTS_DIR/log_analyzer.py" --no-ai
        echo "✅ All checks complete!"
        ;;
    *)
        show_help
        ;;
esac
