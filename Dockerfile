FROM rust

WORKDIR /usr/src/blox
ADD Cargo.lock Cargo.toml ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY . .
RUN cargo build --release
RUN cargo install --locked --path .

CMD ["blox", "serve", "/app"]