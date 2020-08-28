FROM golang:latest

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH "/root/.cargo/bin:$PATH"

WORKDIR /go/app/

# Until we published aias-core repo, wo must clone it manually.
# RUN git clone https://github.com/project-aias/aias-core.git
ADD ./aias-core/core ./core
RUN cd core && cargo build
RUN cp core/target/debug/libaias_core.so core/

RUN go get "github.com/julienschmidt/httprouter"

ADD . /go/app/

ENTRYPOINT [ "./run.sh" ]