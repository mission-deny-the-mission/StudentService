FROM rust:latest

WORKDIR /usr/arc/StudentService
COPY . .
RUN cargo build
CMD ["cargo", "run"]

EXPOSE 8000