FROM docker.io/alpine:edge as builder
RUN apk add --no-cache \
      cargo \
      build-base \
      sqlite-dev \
      openssl-dev
RUN cargo install --no-default-features --features sqlite diesel_cli
COPY . /src
WORKDIR /src
RUN cargo build --release


FROM docker.io/alpine:edge
RUN apk add --no-cache \
      sqlite-libs \
      libgcc \
      libssl1.1 \
 && mkdir -p /opt/upowdb
WORKDIR /opt/upowdb
COPY --from=builder /src/target/release/upowdb-backend /usr/local/bin/upowdb-backend
COPY --from=builder /src/migrations /opt/upowdb/migrations
COPY --from=builder /root/.cargo/bin/diesel /usr/local/bin/diesel
COPY docker-run.sh /opt/upowdb/run.sh
CMD ["/opt/upowdb/run.sh"]
