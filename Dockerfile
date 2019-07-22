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
COPY --from=builder /src/target/release/udb-back /usr/local/bin/udb-back
CMD ["/usr/local/bin/udb-back"]