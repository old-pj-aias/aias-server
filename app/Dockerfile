FROM golang:latest

# Install Cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH "/root/.cargo/bin:$PATH"

WORKDIR /go/app/

# Generate key pair
RUN mkdir keys
RUN openssl genrsa -out ./keys/id_rsa \
    && openssl rsa -in ./keys/id_rsa -pubout -out ./keys/id_rsa.pub \
    && openssl genrsa -out ./keys/judge_id_rsa \
    && openssl rsa -in ./keys/judge_id_rsa -pubout -out ./keys/judge_id_rsa.pub

ADD ./aias-core ./aias-core

RUN go get "github.com/julienschmidt/httprouter"
RUN go get "gorm.io/gorm"
RUN go get "gorm.io/driver/sqlite"

ADD . /go/app/

ENTRYPOINT [ "./run.sh" ]
