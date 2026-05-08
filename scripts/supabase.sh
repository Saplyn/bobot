#!/bin/bash
set -euo pipefail

supabase() {
    bunx supabase --network-id local-network "$@"
}

case "$1" in
"restart")
    supabase stop
    supabase start
    ;;
"gentype" | "typegen" | "typed")
    supabase gen types --linked \
        --schema public \
        --schema graphql_public \
        --schema auth
    ;;
esac

supabase "$@"
