// $ maelstrom test -w echo --bin target/release/examples/echo --time-limit 10
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;

use async_trait::async_trait;
use tracing::warn;

use async_maelstrom::msg::Body::Workload;
use async_maelstrom::msg::Echo;
use async_maelstrom::msg::{Msg, MsgId};
use async_maelstrom::process::{ProcNet, Process};
use async_maelstrom::{Id, Status};

/// Echo server
///
/// The server will run until the runtime shuts it down.
/// It will echo all valid echo requests, and ignore other messages.
#[derive(Default)]
pub struct EchoServer {
    args: Vec<String>,
    net: ProcNet<Echo, ()>,
    id: Id,
    ids: Vec<Id>,
    msg_id: AtomicU64,
}

impl EchoServer {
    fn next_msg_id(&self) -> MsgId {
        self.msg_id.fetch_add(1, SeqCst)
    }
}

#[async_trait]
impl Process<Echo, ()> for EchoServer {
    fn init(
        &mut self,
        args: Vec<String>,
        net: ProcNet<Echo, ()>,
        id: Id,
        ids: Vec<Id>,
        start_msg_id: MsgId,
    ) {
        self.args = args;
        self.net = net;
        self.id = id;
        self.ids = ids;
        self.msg_id = AtomicU64::new(start_msg_id)
    }

    async fn run(&self) -> Status {
        loop {
            // Respond to all echo messages with an echo_ok message echoing the `echo` field
            match self.net.rxq.recv().await {
                Ok(Msg {
                    src,
                    body: Workload(Echo::Echo { msg_id, echo }),
                    ..
                }) => {
                    self.net
                        .txq
                        .send(Msg {
                            src: self.id.clone(),
                            dest: src,
                            body: Workload(Echo::EchoOk {
                                in_reply_to: msg_id,
                                msg_id: Some(self.next_msg_id()),
                                echo,
                            }),
                        })
                        .await?;
                }
                Err(_) => return Ok(()), // Runtime is shutting down.
                Ok(msg) => warn!("received and ignoring an unexpected message: {:?}", msg),
            };
        }
    }
}
