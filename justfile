# Run the bot. Pass -n to also join the Minecraft server's Docker network
# (needed when the server runs in Docker rather than on the host; requires
# MINECRAFT_NETWORK in .env).
run *flags:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ "{{flags}}" == *"-n"* ]]; then
        docker compose -f docker-compose.yml -f docker-compose.network.yml up -d
    else
        docker compose up -d
    fi

# Rebuild the bot's image from source and restart the container. Use this
# after code changes, since `just run` alone won't pick them up. Pass -n to
# also join the Minecraft server's Docker network (see `run`).
rebuild *flags:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ "{{flags}}" == *"-n"* ]]; then
        docker compose -f docker-compose.yml -f docker-compose.network.yml up -d --build
    else
        docker compose up -d --build
    fi
