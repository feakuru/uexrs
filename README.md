# uexrs | WIP

`uexrs` (_Universal EXchange RuSt_) is an AMQP 1.0-compliant multimodal node written in Rust: it can be configured as a replicator, a pubsub exchange, a procedure calling system and potentially other things. It implements only the 1.0.0 version of AMQP using only TCP sockets as the transport.

In terms of the [AMQP specification](https://www.amqp.org/sites/amqp.org/files/amqp.pdf), an instance of `uexrs` is a **Node** (See 2.1 Transport). Whatever you run the instance on (your PC, a VPS, a Kubernetes cluster etc.) is thus a **Container**, and it has the responsibility to manage the amount, lifetime and configuration of your desired Nodes. Thus, `uexrs` concerns itself with passing **Frames** between various clients that connect to it. The steps for handling a new connection are described below; the next section concerns itself with the configuration of a `uexrs` instance; then the operation modes are covered in a bit more depth; then the additional functionality is discussed. Lastly, the `uexrs` web interface is explained.

## How connections are handled

When an incoming TCP connection is established, `uexrs`:

0. Performs the AMQP header exchange and drops the socket if the client sent an unsupported protocol version (which is any but 1.0.0).

1. Splits the socket into a reader and a writer, which are each used to establish an asynchronous **Terminus** (a **Source** and a **Target**, respectively) handler. These handlers would be considered **Links** in terms of AMQP documentation.

The terminus handlers then process each **Frame** that they receive from the client or the internal frame bus and send it to the frame bus if and however needed.

## Configuration

TODO

## AMQP node operation modes

TODO

## Additional functionality

TODO (procedure calling, topics, custom handlers)

## Web interface

TODO

## Contributing

TODO

Note: this project is in the beginning phase of development.

## Useful links

* [AMQP 1.0 specification](https://www.amqp.org/sites/amqp.org/files/amqp.pdf)
* [AMQP 1.0 interactive type reference](https://qpid.apache.org/amqp/type-reference.html)
