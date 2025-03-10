// $ maelstrom test -w echo --bin target/release/examples/echo --time-limit 10
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;

use async_trait::async_trait;

use async_maelstrom::msg::Body::Workload;
use async_maelstrom::msg::{Key, LinKv};
use async_maelstrom::msg::{Msg, MsgId};
use async_maelstrom::process::{ProcNet, Process};
use async_maelstrom::{Error, Id, Status, Val};

/// Echo server
///
/// The server will run until the runtime shuts it down.
/// It will echo all valid echo requests, and ignore other messages.
#[derive(Default)]
pub struct SimpleKVServer {
    args: Vec<String>,
    net: ProcNet<LinKv, ()>,
    id: Id,
    ids: Vec<Id>,
    msg_id: AtomicU64,
}

impl SimpleKVServer {
    fn next_msg_id(&self) -> MsgId {
        self.msg_id.fetch_add(1, SeqCst)
    }
}

#[async_trait]
impl Process<LinKv, ()> for SimpleKVServer {
    fn init(
        &mut self,
        args: Vec<String>,
        net: ProcNet<LinKv, ()>,
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
                    dest: _dest,
                    body: Workload(LinKv::Read { msg_id, key }),
                    ..
                }) => self.handle_read(msg_id, key, src).await?,

                Ok(Msg {
                    src,
                    dest: _dest,
                    body: Workload(LinKv::Write { msg_id, key, value }),
                    ..
                }) => self.handle_write(msg_id, key, value, src).await?,

                Ok(Msg {
                    src,
                    dest: _dest,
                    body:
                        Workload(LinKv::Cas {
                            msg_id,
                            key,
                            from,
                            to,
                        }),
                    ..
                }) => self.handle_cas(msg_id, key, from, to, src).await?,

                Err(_) => return Ok(()), // Runtime is shutting down.
                Ok(msg) => {
                    tracing::warn!("unexpected message: {msg:?}");
                    return Err(Error::UnexpectedMsg { expected: "idk" });
                }
            };
        }
    }
}

impl SimpleKVServer {
    async fn handle_read(&self, msg_id: MsgId, key: Key, src: Id) -> Status {
        Ok(())
    }

    async fn handle_write(&self, msg_id: MsgId, key: Key, value: Val, src: Id) -> Status {
        Ok(())
    }

    async fn handle_cas(&self, msg_id: MsgId, key: Key, from: Val, to: Val, src: Id) -> Status {
        Ok(())
    }
}
