FROM golang:1.23.4-alpine AS builder

WORKDIR /app

COPY go.mod go.sum ./

RUN go mod download && go mod verify

COPY . .

RUN go build -o mnstr main.go

FROM alpine:3.19.0

COPY --from=builder /app/mnstr /usr/local/bin/mnstr

EXPOSE 8080

CMD ["mnstr"]