FROM rust:1.49.0-alpine3.12 AS builder

WORKDIR /home
COPY . .
RUN ls -la && \
    apk --update add --no-cache --virtual .deps gcc musl-dev openssl-dev && \
    cargo build --release && \
    apk del .deps

FROM jrottenberg/ffmpeg:4.3.1-alpine311
WORKDIR /home
COPY --from=builder /home/target/release/writer_rs ./
ENTRYPOINT ["/home/writer_rs"]
CMD ["rtsp://admin:iotcamera64@185.253.216.24:12249/cam/realmonitor?channel=1&subtype=0", "605e7ef7-771c-45a5-bc21-a83090954ff4"]
