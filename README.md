<div align="center">

# TCP MULTIPLEXER

_A Single-threaded Asyncronous TCP Multiplexer._

## how it works

### Example: connecting a client using `netcat`

</div>

First, ensure that your TCP Multiplexer server is running. You can start the server by executing the main function in your Rust application:

```bash
cargo run -- server
```

Open a new terminal window and use the following command to connect to the server using `nc`:

```
nc -v 127.0.0.1 27632
```

_Note: you can connect as many clients as you want to broadcast and visualize the messages._
