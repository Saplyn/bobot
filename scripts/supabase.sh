#!/bin/bash

supabase() {
  bunx supabase --network-id local-network "$@"
}

case "$1" in
"restart")
  supabase stop
  supabase start
  exit 0
  ;;
esac

supabase "$@"
