use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio::sync::broadcast::error::RecvError;
use tracing::info;

use crate::commands::server::Message;
use crate::socket::wrapper::OwnedWriteHalf;
use crate::socket::wrapper::SocketAddr;
use crate::socket::wrapper::Stream;

/// # Handle Incoming Messages
///
/// This asynchronous function handles messages from a network stream and a
/// broadcast channel. It reads data from the stream, writes acknowledgments,
/// and broadcasts messages to other clients.
///
/// ## Parameters
///
/// - `stream`: The network stream to read from and write to (`Stream`).
/// - `tx`: A broadcast channel sender for sending messages
///   (`broadcast::Sender<Message>`).
/// - `addr`: The socket address of the connected client (`SocketAddr`).
///
/// ## Returns
///
/// A result indicating success (`miette::Result<()>`) or an error if something
/// goes wrong.
///
/// ## Example
///
/// ```rust
/// // Example usage in an asynchronous context
/// handle_message(stream, tx, addr).await?;
/// ```
pub async fn handle_message(
    stream: Stream,
    tx: broadcast::Sender<Message>,
    addr: SocketAddr,
) -> miette::Result<()> {
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    let login_acknowledgment = &format!("LOGIN: {}\n", addr.port().unwrap());
    writer.write_all(login_acknowledgment.as_bytes()).await.expect("write failed");
    writer.flush().await.expect("flush failed");

    let mut rx = tx.subscribe();
    let mut incoming = String::new();
    loop {
        let tx = tx.clone();
        tokio::select! {
            // Read from broadcast channel.
            result = rx.recv() => {
                read_from_broadcast_channel(result, addr.clone(), &mut writer ).await?;
            }

            // Read from socket.
            network_read_result = reader.read_line(&mut incoming) => {
                let num_bytes_read: usize = network_read_result.expect("read failed");
                // EOF check.
                if num_bytes_read == 0 {
                    break;
                }

                read_stream(num_bytes_read, &incoming, &mut writer, tx, addr.clone()).await?;
                incoming.clear();
            }
        }
    }

    Ok(())
}

/// # Read from Broadcast Channel
///
/// This asynchronous function reads messages from a broadcast channel and
/// writes them to a network stream, excluding messages sent by the same client.
///
/// ## Parameters
///
/// - `result`: The result from receiving a message from the broadcast channel
///   (`Result<Message, RecvError>`).
/// - `addr`: The socket address of the connected client (`SocketAddr`).
/// - `writer`: A mutable reference to a buffered writer for the network stream
///   (`&mut BufWriter<OwnedWriteHalf>`).
///
/// ## Returns
///
/// A result indicating success (`miette::Result<()>`) or an error if something
/// goes wrong.
///
/// ## Example
///
/// ```rust
/// // Example usage in an asynchronous context
/// read_from_broadcast_channel(result, addr, &mut writer).await?;
/// ```
async fn read_from_broadcast_channel(
    result: Result<Message, RecvError>,
    addr: SocketAddr,
    writer: &mut BufWriter<OwnedWriteHalf>,
) -> miette::Result<()> {
    match result {
        Ok(it) => {
            let msg: Message = it;
            if msg.addr.port().unwrap() != addr.port().unwrap() {
                writer.write_all(msg.payload.as_bytes()).await.expect("write failed");
                writer.flush().await.expect("flush failed");
            }
        }
        Err(error) => {
            info!("[{}]: channel error: {:?}", addr.port().unwrap(), error);
        }
    }

    Ok(())
}

/// # Read from Network Stream
///
/// This asynchronous function processes incoming data from the network stream,
/// sends it to the broadcast channel, and writes acknowledgments back to the
/// client.
///
/// ## Parameters
///
/// - `num_bytes_read`: The number of bytes read from the network stream
///   (`usize`).
/// - `incoming`: The incoming message as a string slice (`&str`).
/// - `writer`: A mutable reference to a buffered writer for the network stream
///   (`&mut BufWriter<OwnedWriteHalf>`).
/// - `tx`: A broadcast channel sender for sending messages (`Sender<Message>`).
/// - `addr`: The socket address of the connected client (`SocketAddr`).
///
/// ## Returns
///
/// A result indicating success (`miette::Result<()>`) or an error if something
/// goes wrong.
///
/// ## Example
///
/// ```rust
/// // Example usage in an asynchronous context
/// read_stream(num_bytes_read, incoming, &mut writer, tx, addr).await?;
/// ```
async fn read_stream(
    num_bytes_read: usize,
    incoming: &str,
    writer: &mut BufWriter<OwnedWriteHalf>,
    tx: Sender<Message>,
    addr: SocketAddr,
) -> miette::Result<()> {
    info!(
        "[{}]: incoming: {}, size: {}",
        addr.port().unwrap(),
        incoming.trim(),
        num_bytes_read
    );

    let outgoing = handle_incoming_message(incoming);

    // Broadcast outgoing to the channel.
    let _ = tx.send(Message {
        addr: addr.clone(),
        payload: handle_forward_message(addr.port().unwrap(), outgoing.to_string()),
        from: addr.port().unwrap().to_string(),
    });

    info!(
        "[{}]: outgoing: {}, size: {}",
        addr.port().unwrap(),
        outgoing.trim(),
        num_bytes_read
    );

    let acknowledgment = handle_acknowledgment_message();
    writer.write_all(acknowledgment.as_bytes()).await.expect("write failed");
    writer.flush().await.expect("flush failed");

    Ok(())
}

/// # Handle Incoming Message
///
/// Converts an incoming message to uppercase.
///
/// ## Parameters
///
/// - `incoming`: The incoming message as a string slice (`&str`).
///
/// ## Returns
///
/// A `String` representing the uppercase version of the incoming message.
///
/// ## Example
///
/// ```rust
/// let result = handle_incoming_message("hello");
/// assert_eq!(result, "HELLO");
/// ```
fn handle_incoming_message(incoming: &str) -> String { incoming.to_uppercase() }

/// # Handle Acknowledgment Message
///
/// Generates a standard acknowledgment message.
///
/// ## Returns
///
/// A `String` representing the acknowledgment message.
///
/// ## Example
///
/// ```rust
/// let acknowledgment = handle_acknowledgment_message();
/// assert_eq!(acknowledgment, "ACK:MESSAGE\n");
/// ```
fn handle_acknowledgment_message() -> String { "ACK:MESSAGE\n".to_string() }

/// # Handle Forward Message
///
/// Formats a message for forwarding, including the sender's port.
///
/// ## Parameters
///
/// - `port`: The port number of the sender (`u16`).
/// - `outgoing`: The outgoing message (`String`).
///
/// ## Returns
///
/// A `String` formatted for forwarding, including the port and message.
///
/// ## Example
///
/// ```rust
/// let forwarded = handle_forward_message(8080, "REQUEST".to_string());
/// assert_eq!(forwarded, "MESSAGE:8080 REQUEST");
/// ```
fn handle_forward_message(port: u16, outgoing: String) -> String {
    format!("MESSAGE:{} {}", port, outgoing)
}
