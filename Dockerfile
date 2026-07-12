FROM rust:1.97-bookworm AS rust-builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim AS trumpscript-source
ARG TRUMPSCRIPT_COMMIT=3793b905925b55c0296b066586c7d612c7220ca0
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git \
    && rm -rf /var/lib/apt/lists/* \
    && git init /opt/trumpscript \
    && cd /opt/trumpscript \
    && git remote add origin https://github.com/samshadwell/TrumpScript.git \
    && git fetch --depth 1 origin "$TRUMPSCRIPT_COMMIT" \
    && git checkout --detach FETCH_HEAD \
    && rm -rf .git \
    && sed -i \
       -e 's/return Module(body=body_list)/return Module(body=body_list, type_ignores=[])/' \
       -e 's/Num(/Constant(value=/g' \
       -e 's/Str(/Constant(value=/g' \
       -e 's/NameConstant(/Constant(/g' \
       src/trumpscript/parser.py

FROM r-base:4.6.1
ARG APP_VERSION=1.0.3
LABEL org.opencontainers.image.title="Crazy Race Game" \
      org.opencontainers.image.description="Server-rendered Rust, R, TrumpScript and Piet racing game" \
      org.opencontainers.image.version="${APP_VERSION}" \
      org.opencontainers.image.source="https://github.com/Pepitodrop/CrazyRaceGame" \
      org.opencontainers.image.documentation="https://github.com/Pepitodrop/CrazyRaceGame/blob/main/docs/PRODUCTION.md" \
      org.opencontainers.image.licenses="MIT"
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates python3 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 racer \
    && useradd --uid 10001 --gid 10001 --create-home --shell /usr/sbin/nologin racer
COPY --from=rust-builder /build/target/release/crazy-race-game /usr/local/bin/crazy-race-game
COPY --from=trumpscript-source /opt/trumpscript /opt/trumpscript
COPY scripts ./scripts
COPY announcer ./announcer
COPY piet ./piet
COPY data ./data
RUN chmod 0555 /usr/local/bin/crazy-race-game /opt/trumpscript/bin/TRUMP \
    && /opt/trumpscript/bin/TRUMP --shut-up /app/announcer/race.tr >/tmp/trumpscript-check.txt \
    && test -s /tmp/trumpscript-check.txt \
    && rm /tmp/trumpscript-check.txt
ENV BIND_ADDRESS=0.0.0.0:8080 \
    TRACK_SEED=0 \
    MAX_CONNECTIONS=128 \
    MAX_ROOMS=1000 \
    ROOM_TTL_SECONDS=7200 \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1
EXPOSE 8080
USER 10001:10001
STOPSIGNAL SIGTERM
HEALTHCHECK --interval=15s --timeout=3s --start-period=8s --retries=3 \
  CMD ["crazy-race-game", "--healthcheck"]
CMD ["crazy-race-game"]
