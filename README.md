# minecraft-bot

A Discord bot that manages a Minecraft server over RCON.

## Setup

1. Copy the example env file and fill it in:

   ```
   cp .env.example .env
   ```

   - `DISCORD_TOKEN` — your Discord bot token.
   - `RCON_ADDRESS` — host and port of the Minecraft server's RCON interface.
   - `RCON_PASSWORD` — RCON password.
   - `MINECRAFT_NETWORK` — only needed if the Minecraft server runs in Docker (see below).

2. Run the bot with [`just`](https://github.com/casey/just):

   ```
   just run
   ```

## Reaching the Minecraft server

The bot needs a way to reach the Minecraft server's RCON port. Which setup you use depends on where the server runs.

### Server runs on the host machine (default)

The container reaches the host via `host.docker.internal`, which is set up automatically. Just set:

```
RCON_ADDRESS=host.docker.internal:25575
```

and run:

```
just run
```

### Server runs in another Docker container

Join the same Docker network as the `minecraft-server` container instead. Find its network name:

```
docker inspect minecraft-server --format '{{json .NetworkSettings.Networks}}'
```

Set `MINECRAFT_NETWORK` in `.env` to that network name, and `RCON_ADDRESS` to the container name (e.g. `minecraft-server:25575`). Then run with the `-n` flag to join that network:

```
just run -n
```

This uses `docker-compose.network.yml` as an overlay on top of the base `docker-compose.yml`, so the network is only attached when explicitly requested.
