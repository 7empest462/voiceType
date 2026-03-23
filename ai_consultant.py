#!/usr/bin/env python3
"""
AI Code Consultant
Pipe code or errors into this tool for instant AI analysis.

Usage:
  cat raw_code.py | python ai_consultant.py [mode]
  
Modes:
  --explain (default): Explain what the code does
  --fix: Fix bugs or errors in the input
  --refactor: Improve code quality and readability
  --test: Generate unit tests for the code
"""

import sys
import argparse
import requests
import json
import os

OLLAMA_URL = "http://localhost:11434/api/generate"
MODEL = "qwen2.5-coder:7b"

# Prompts for different modes
PROMPTS = {
    "explain": "Explain this code in simple terms. Describe its purpose, logic, and any potential issues.",
    "fix": "Fix any bugs, errors, or security issues in this code. Output ONLY the fixed code block, then a brief explanation of fixes.",
    "refactor": "Refactor this code to follow best practices, improve readability, and optimize performance. Output ONLY the refactored code.",
    "test": "Generate comprehensive unit tests for this code using standard testing libraries. Output ONLY the test code."
}

def query_ollama(prompt, context):
    """Send query to Ollama."""
    full_prompt = f"{prompt}\n\nCode/Input:\n```\n{context}\n```\n\nResponse:"
    
    try:
        response = requests.post(
            OLLAMA_URL,
            json={
                "model": MODEL,
                "prompt": full_prompt,
                "stream": False,
                "options": {"temperature": 0.2} 
            },
            timeout=120
        )
        if response.status_code == 200:
            return response.json().get('response', '').strip()
        else:
            return f"Error: Ollama API returned {response.status_code}"
    except Exception as e:
        return f"Error connecting to Ollama: {e}"

def main():
    parser = argparse.ArgumentParser(description="AI Code Consultant")
    parser.add_argument('--explain', action='store_true', help='Explain the code (default)')
    parser.add_argument('--fix', action='store_true', help='Fix bugs in the code')
    parser.add_argument('--refactor', action='store_true', help='Refactor the code')
    parser.add_argument('--test', action='store_true', help='Generate unit tests')
    
    args = parser.parse_args()
    
    # Determine mode
    mode = "explain"
    if args.fix: mode = "fix"
    elif args.refactor: mode = "refactor"
    elif args.test: mode = "test"
    
    # Read stdin
    if sys.stdin.isatty():
        print("🤖 AI Code Consultant")
        print("Usage: cat file.py | ai_consultant.py [--fix|--refactor|--test]")
        print("Waiting for input (Ctrl+D to finish)...")
        input_data = sys.stdin.read()
    else:
        input_data = sys.stdin.read()
        
    if not input_data.strip():
        print("❌ No input provided.")
        return

    print(f"\n🧠 Analyzing ({mode})...\n")
    
    result = query_ollama(PROMPTS[mode], input_data)
    
    # Print result with nice formatting
    print("-" * 60)
    print(result)
    print("-" * 60 + "\n")

if __name__ == "__main__":
    main()
