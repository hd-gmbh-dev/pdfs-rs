# image: pdfs-service
ARG RUST_IMAGE=rust:1.89-alpine
ARG BASE_IMAGE=alpine:3.24

FROM ${RUST_IMAGE} AS build
# the official rust:alpine image ships no C toolchain; no-op where it is present
RUN apk add --no-cache musl-dev gcc
WORKDIR /src
COPY . .
RUN cargo build --release --bin pdfs-service

FROM ${BASE_IMAGE}
COPY --from=build /src/target/release/pdfs-service /usr/bin/pdfs-service
RUN addgroup -S pdfs && adduser -S pdfs -G pdfs -u 1000
USER pdfs
EXPOSE 11242
CMD ["/usr/bin/pdfs-service"]
