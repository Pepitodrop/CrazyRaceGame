FROM rust:1.82-bookworm AS rust-builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim AS trumpscript-source
ARG TRUMPSCRIPT_COMMIT=3793b905925b55c0296b066586c7d612c7220ca0
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git \
    && rm -rf /var/lib/apt/lists/* \
    && git clone https://github.com/samshadwell/TrumpScript.git /opt/trumpscript \
    && cd /opt/trumpscript \
    && git checkout "$TRUMPSCRIPT_COMMIT" \
    && rm -rf .git \
    && sed -i 's/return Module(body=body_list)/return Module(body=body_list, type_ignores=[])/' src/trumpscript/parser.py

FROM r-base:4.4.2
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates python3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 racer
COPY --from=rust-builder /build/target/release/crazy-race-game /usr/local/bin/crazy-race-game
COPY --from=trumpscript-source /opt/trumpscript /opt/trumpscript
COPY scripts ./scripts
COPY announcer ./announcer
COPY piet ./piet
COPY data ./data
RUN chmod +x /opt/trumpscript/bin/TRUMP \
    && /opt/trumpscript/bin/TRUMP --shut-up /app/announcer/race.tr >/tmp/trumpscript-check.txt \
    && test -s /tmp/trumpscript-check.txt \
    && rm /tmp/trumpscript-check.txt
ENV BIND_ADDRESS=0.0.0.0:8080 \
    TRACK_SEED=42
EXPOSE 8080
USER racer
HEALTHCHECK --interval=15s --timeout=3s --start-period=8s --retries=3 \
  CMD ["crazy-race-game", "--healthcheck"]
CMD ["crazy-race-game"]
