FROM rustlang/rust:nightly

RUN mkdir /app
WORKDIR /app

COPY . .

RUN cargo +nightly build --release

CMD ./target/release/tokyo-server
