<div align="center">

# TCP MULTIPLEXER

_A Single-threaded Asyncronous TCP Multiplexer._

### Example: connecting a client using `netcat`

</div>

```bash
git clone git@github.com:falcucci/tcp-multiplexer.git
```

### Running the server

```bash
cargo run -- server
```

Open a new terminal window and use the following command to connect to the server using `nc`:

```
nc -v 127.0.0.1 27632
```

_Note: you can connect as many clients as you want to broadcast and visualize the messages._
