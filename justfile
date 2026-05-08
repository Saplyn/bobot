set unstable # enable user-defined fn

project_root := justfile_directory()
workers := "oauth|spirit|labour"
pages := "online"
supabase := "db|database|supabase|spb"

throw(msg) := f"""
    echo -e \"{{BG_RED}}{{BOLD}}  {{msg}} {{NORMAL}}\"
    exit 1
"""
log(cmd) := f"echo -e {{MAGENTA}}$ {{cmd}}{{NORMAL}}"
run(cmd) := f"""
    {{log(cmd)}}
    {{cmd}}
"""

[arg("project", pattern="""
    |oauth|labour|spirit
    |online
    |db|database|supabase|spb
""")]
[no-cd]
dev project="":
    #!/usr/bin/env bash
    set -euo pipefail

    dev_worker() {
        wrangler dev
    }
    dev_page() {
        bun run dev
    }
    supabase() {
        bunx supabase --network-id local-network "$@"
    }

    case "{{ project }}" in
        {{ workers }})
            cd {{ project_root }}/bobot/{{ project }}
            dev_worker
            ;;
        {{ pages }})
            cd {{ project_root }}/bobot/{{ project }}
            dev_page
            ;;
        {{ supabase }})
            {{ run("supabase start") }}
            ;;
        "")
            if [ -e "wrangler.toml" ] || [ -e "wrangler.jsonc" ]; then
                dev_worker
                exit 0
            fi
            if [ -e "nuxt.config.ts" ]; then
                dev_page
                exit 0
            fi
            {{ throw("Not inside any dev-able sub-project, specify 'db' for local database") }}
            ;;
    esac

[arg("project", pattern="""
    oauth|labour|spirit
    |online
    |db|database|supabase|spb
""")]
[no-cd]
start project="db":
    @just dev {{ project }}

stop:
    #!/usr/bin/env bash
    {{ run("supabase stop") }}

gentype:
    #!/usr/bin/env bash

    db_types_path="{{ project_root }}/bobot/online/app/types/database.types.ts"
    mkdir -p "$(dirname "$db_types_path")"

    {{ log("supabase gen types") }}
    supabase gen types --linked \
        --schema public \
        --schema graphql_public \
        --schema auth \
        >$db_types_path
    exit_code=$?

    if [ $exit_code -ne 0 ]; then
        {{ throw("Failed to generate supabase types") }}
    fi
    echo "Supabase types generated at $db_types_path"

genkey tag="":
    #!/usr/bin/env bash
    lyn_key=$(openssl rand -base64 33)

    if [ -z {{ tag }} ]; then
        echo lyn_"$lyn_key"
    else
        echo lyn_{{ tag }}_"$lyn_key"
    fi

update:
    #!/usr/bin/env bash

    {{ run("cargo update") }}
    {{ run("bun update") }}

outdated:
    #!/usr/bin/env bash

    {{ run("cargo outdated") }}
    {{ run("bun outdated") }}

audit:
    #!/usr/bin/env bash

    {{ run("cargo audit --deny warnings") }}
    cargo_exit=$?

    {{ run("bun audit --audit-level=high") }}
    bun_exit=$?

    if [ $cargo_exit -ne 0 ] || [ $bun_exit -ne 0 ]; then
        {{ throw("Not all audit is passed") }}
    fi
