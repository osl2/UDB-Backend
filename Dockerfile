FROM docker.io/alpine:edge as builder
RUN apk add --no-cache \
      cargo \
      build-base \
      sqlite-dev \
      postgresql-dev \
      openssl-dev
COPY . /src
WORKDIR /src
ARG DATABASE_BACKEND=sqlite
RUN cargo build --release --no-default-features --features ${DATABASE_BACKEND}


FROM docker.io/alpine:edge
RUN apk add --no-cache \
      sqlite-libs \
      postgresql-libs \
      libgcc \
      libssl1.1 \
 && mkdir -p /opt/upowdb
WORKDIR /opt/upowdb
COPY --from=builder /src/target/release/upowdb-backend /usr/local/bin/upowdb-backend
COPY docker-run.sh /opt/upowdb/run.sh
CMD ["/opt/upowdb/run.sh"]
