FROM golang:1.22

ADD /go /app/go
ADD /demo-iota/go /app/demo-iota/go

WORKDIR /app/demo-iota/go/
RUN go mod tidy

RUN go build -o /iota
CMD ["/iota"]

