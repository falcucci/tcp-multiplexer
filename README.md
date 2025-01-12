<div align="center">

# TCP MULTIPLEXER

_A Single-threaded Asyncronous TCP Multiplexer._

</div>

<div align="center">
    <img src="https://github.com/user-attachments/assets/ec69466a-696a-440f-a3d2-09b702213e9e" alt="Screenshot of the network">
    <em>Screenshot of the network sampling connected peers.</em>
</div>

### Running the server

```bash
git clone git@github.com:falcucci/tcp-multiplexer.git
```

```bash
cargo run -- server
```

And you should see the following output while connection peers:

```bash
INFO tcp_multiplexer::commands::server: listening socket=Tcp(127.0.0.1, 27632)
INFO tcp_multiplexer::commands::server: client connected socket=Ip(127.0.0.1:62008)
INFO tcp_multiplexer::commands::server: client connected socket=Ip(127.0.0.1:62009)
INFO tcp_multiplexer::commands::server: client connected socket=Ip(127.0.0.1:62010)
```

### Example: connecting a client using `netcat`

Open a new terminal window and use the following command to connect to the server using `nc`:

```bash
# alice
nc -v 127.0.0.1 27632
```

```bash
# bob
nc -v 127.0.0.1 27632
```

```bash
# john
nc -v 127.0.0.1 27632
```

After the connection is stablished, prompt the message you want to broadcast.

```bash
Connection to 127.0.0.1 port 27632 [tcp/*] succeeded!
LOGIN: 62107
REQUEST
ACK:MESSAGE
MESSAGE:62090 REPLY
```

In case you use [Zellij](https://zellij.dev/documentation/overview) and want to automatize its setup, there is a [layout](./layout/zellij.kdl) available:

```bash
layout {
  pane name="tcp-multiplexer-server" {
    command "cargo"
    args "run" "--" "server"
  }
  pane split_direction="vertical" {
    pane   name="alice" start_suspended=true {
      command "nc"
      args "-v" "127.0.0.1" "27632"
    }
    pane name="bob" start_suspended=true {
      command "nc"
      args "-v" "127.0.0.1" "27632"
    }
    pane name="john" start_suspended=true {
      command "nc"
      args "-v" "127.0.0.1" "27632"
    }
  }
}
```

And then just start zellij using:

```bash
zellij --layout ./layout/zellij.kdl
```

_Note: you can connect as many clients as you want to broadcast and visualize the messages._
