#!/bin/bash

source "$PROJECT_ROOT"/scripts/color.lib.sh

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
