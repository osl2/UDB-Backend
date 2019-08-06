FROM docker.io/alpine as builder
COPY . /src
RUN apk add --no-cache \
      cargo \
      build-base \
      sqlite-dev \
      postgresql-dev \
 && cd /src \
 && cargo build --release


FROM docker.io/alpine
COPY --from=builder /src/target/release/upowdb-backend /usr/local/bin/upowdb-backend
CMD ["/usr/local/bin/upowdb-backend"]
