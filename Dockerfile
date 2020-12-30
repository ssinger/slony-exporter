from rust:1.48-buster as build
COPY Cargo.toml /
COPY src /src
RUN cargo build --release

from debian:buster
RUN apt-get update && apt-get install libssl1.1
COPY --from=build target/release/slony-exporter /slony-exporter
EXPOSE 9090
ENV PORT 9090
ENV SLONY_CLUSTER=test
ENV POSTGRES_URL=postgresql://postgres@localhost:5432/test
USER nobody
ENTRYPOINT /slony-exporter
