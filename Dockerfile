# image: harbor.akdb.de/shapth/service/pdfs-service (local builds: `pdfs-service:local`)
ARG RUST_IMAGE=harbor.akdb.de/ozg/docker/rust-alpine:1.97.1
ARG BASE_IMAGE=harbor.akdb.de/ozg/docker/alpine:3.24.0

FROM ${RUST_IMAGE} AS build
WORKDIR /src
COPY . .
RUN cargo build --release --bin pdfs-service

FROM ${BASE_IMAGE}
COPY --from=build /src/target/release/pdfs-service /usr/bin/pdfs-service
RUN addgroup -S pdfs && adduser -S pdfs -G pdfs -u 1000
USER pdfs
EXPOSE 11242
CMD ["/usr/bin/pdfs-service"]
