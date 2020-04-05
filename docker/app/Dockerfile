FROM clux/muslrust:latest AS builder
WORKDIR /satelit-scheduler/
COPY . .
RUN docker/app/scripts/build-project.sh

FROM alpine:latest
WORKDIR /satelit-scheduler/
EXPOSE 9060
COPY --from=builder /satelit-scheduler/satelit-scheduler.tar.gz /satelit-scheduler/
RUN ["tar", "-xvf", "satelit-scheduler.tar.gz"]
ENTRYPOINT ["/satelit-scheduler/entry.sh"]
HEALTHCHECK --interval=1m --start-period=10s CMD ps cax | grep satelit-scheduler >/dev/null || exit 1
