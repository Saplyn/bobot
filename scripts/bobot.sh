#!/bin/bash

RESET="\033[0m"

BLACK="\033[30m"
RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
BLUE="\033[34m"
MAGENTA="\033[35m"
CYAN="\033[36m"
WHITE="\033[37m"

BRIGHT_BLACK="\033[90m"
BRIGHT_RED="\033[91m"
BRIGHT_GREEN="\033[92m"
BRIGHT_YELLOW="\033[93m"
BRIGHT_BLUE="\033[94m"
BRIGHT_MAGENTA="\033[95m"
BRIGHT_CYAN="\033[96m"
BRIGHT_WHITE="\033[97m"

BOLD="\033[1m"
DIM="\033[2m"
ITALIC="\033[3m"
UNDERLINE="\033[4m"
REVERSED="\033[7m"
HIDDEN="\033[8m"
STRIKETHROUGH="\033[9m"

BG_BLACK="\033[40m"
BG_RED="\033[41m"
BG_GREEN="\033[42m"
BG_YELLOW="\033[43m"
BG_BLUE="\033[44m"
BG_MAGENTA="\033[45m"
BG_CYAN="\033[46m"
BG_WHITE="\033[47m"

BG_BRIGHT_BLACK="\033[100m"
BG_BRIGHT_RED="\033[101m"
BG_BRIGHT_GREEN="\033[102m"
BG_BRIGHT_YELLOW="\033[103m"
BG_BRIGHT_BLUE="\033[104m"
BG_BRIGHT_MAGENTA="\033[105m"
BG_BRIGHT_CYAN="\033[106m"
BG_BRIGHT_WHITE="\033[107m"

supabase() {
  "$PROJECT_ROOT"/scripts/supabase.sh "$@"
}

bobot_help() {
  echo "Supported commands:"
  echo "  typed        - Generate Supabase types"
  echo "  pre-commit   - Run pre-commit checks"
  echo "  apikey       - Generate apikey"
}

bobot_typed() {
  local db_types_path="$PROJECT_ROOT"/bobot/online/app/types/database.types.ts
  mkdir -p "$(dirname "$db_types_path")"

  echo -e -n "$BRIGHT_BLACK"
  supabase typed >"$db_types_path"
  local exit_code=$?
  echo -e -n "$RESET"
  if [ $exit_code -ne 0 ]; then
    return $exit_code
  fi

  echo "Supabase types generated at $db_types_path"
}

bobot_pre_commit() {
  local db_types_path="$PROJECT_ROOT"/bobot/online/app/types/database.types.ts
  local temp_file
  temp_file="$(mktemp)"

  echo -e -n "$BRIGHT_BLACK"
  supabase typed >"$temp_file"
  local exit_code=$?
  echo -e -n "$RESET"
  if [ $exit_code -ne 0 ]; then
    return $exit_code
  fi

  if ! diff -q "$temp_file" "$db_types_path" >/dev/null; then
    echo -e "$BOLD$BRIGHT_RED"" Supabase types are out of date.$RESET"
    echo "Run \`bobot typed\`, then commit the updated types."
    rm -f "$temp_file"
    exit 1
  fi

  rm -f "$temp_file"
  echo -e "$BRIGHT_GREEN"" Supabase types are up to date.$RESET"
}

bobot_apikey() {
  local lyn_key
  lyn_key=$(openssl rand -base64 33)

  if [ -z "$1" ]; then
    echo lyn_"$lyn_key"
  else
    echo lyn_"$1"_"$lyn_key"
  fi
}

case "$1" in
"typed")
  bobot_typed
  ;;
"pre-commit")
  bobot_pre_commit
  ;;
"apikey")
  bobot_apikey "$2"
  ;;
"" | "help" | "-h" | "--help")
  bobot_help "$@"
  ;;
*)
  echo "Invalid usage: \`bobot $*\`"
  bobot_help "$@"
  exit 1
  ;;
esac
