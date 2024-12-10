FROM docker.io/rust:1.75 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM registry.redhat.io/ubi9/ubi-minimal
COPY --from=builder /usr/src/app/target/release/tcp-info-server /usr/local/bin/
EXPOSE 8080
CMD ["tcp-info-server"]
