# Run the bot. Pass -n to also join the Minecraft server's Docker network
# (needed when the server runs in Docker rather than on the host; requires
# MINECRAFT_NETWORK in .env).
run *flags:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ "{{flags}}" == *"-n"* ]]; then
        docker compose -f docker-compose.yml -f docker-compose.network.yml up -d --build
    else
        docker compose up -d --build
    fi
