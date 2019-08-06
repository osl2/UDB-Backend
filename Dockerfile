FROM docker.io/alpine:edge as builder
COPY . /src
RUN apk add --no-cache \
      cargo \
      build-base \
      sqlite-dev \
      postgresql-dev \
 && cd /src \
 && cargo build --release


FROM docker.io/alpine:edge
COPY --from=builder /src/target/release/upowdb-backend /usr/local/bin/upowdb-backend
RUN apk add --no-cache \
      sqlite-libs \
      postgresql-libs \
      libgcc
CMD ["/usr/local/bin/upowdb-backend"]
