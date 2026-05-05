#!/bin/bash

supabase() {
  bunx supabase --network-id local-network "$@"
}

case "$1" in
"restart")
  supabase stop || exit 1
  supabase start || exit 1
  exit 0
  ;;
"typed")
  supabase gen types --linked \
    --schema public \
    --schema graphql_public \
    --schema auth || exit 1
  exit 0
  ;;
esac

supabase "$@"
