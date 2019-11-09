FROM alpine:3.10
COPY ./target/release/intensifies /bin/intensifies
EXPOSE 80
ENTRYPOINT ["intensifies", "web", "80"]

