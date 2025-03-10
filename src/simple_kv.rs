use std::collections::HashMap;
// $ maelstrom test -w echo --bin target/release/examples/echo --time-limit 10
use std::sync::RwLock;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;

use async_trait::async_trait;

use async_maelstrom::msg::{Body, Error as WError, Key, LinKv, Msg, MsgId};
use async_maelstrom::process::{ProcNet, Process};
use async_maelstrom::{Error, Id, Status, Val};

use derivative::Derivative;

#[derive(Default, Debug)]
struct State {
    kv: HashMap<Key, Val>,
}

#[derive(Default, Derivative)]
#[derivative(Debug)]
pub struct SimpleKVServer {
    args: Vec<String>,
    #[derivative(Debug = "ignore")]
    net: ProcNet<LinKv, ()>,
    id: Id,
    ids: Vec<Id>,
    msg_id: AtomicU64,

    // state. rwlock because the api of the maelstrom lib is wrong (run should take &mut self)
    // TODO: fork it and use that.
    state: RwLock<State>,
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
            match self.net.rxq.recv().await {
                Ok(Msg {
                    src,
                    dest: _dest,
                    body: Body::Workload(LinKv::Read { msg_id, key }),
                    ..
                }) => self.handle_read(msg_id, key, src).await?,

                Ok(Msg {
                    src,
                    dest: _dest,
                    body: Body::Workload(LinKv::Write { msg_id, key, value }),
                    ..
                }) => self.handle_write(msg_id, key, value, src).await?,

                Ok(Msg {
                    src,
                    dest: _dest,
                    body:
                        Body::Workload(LinKv::Cas {
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
    #[tracing::instrument(skip(self), level = "debug")]
    async fn handle_read(&self, msg_id: MsgId, key: Key, src: Id) -> Status {
        let val = {
            let kv = &self.state.read().unwrap().kv;
            kv.get(&key).unwrap_or(&Val::Null).to_owned()
        };
        self.reply(
            src,
            LinKv::ReadOk {
                in_reply_to: msg_id,
                msg_id: Some(self.next_msg_id()),
                value: val,
            },
        )
        .await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    async fn handle_write(&self, msg_id: MsgId, key: Key, value: Val, src: Id) -> Status {
        {
            let kv = &mut self.state.write().unwrap().kv;
            kv.insert(key.clone(), value.clone());
        }
        self.reply(
            src,
            LinKv::WriteOk {
                in_reply_to: msg_id,
            },
        )
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    async fn handle_cas(&self, msg_id: MsgId, key: Key, from: Val, to: Val, src: Id) -> Status {
        #[derive(PartialEq, Eq)]
        enum Res {
            Ok,
            Err20,
            Err22,
        }
        let res = {
            let kv = &mut self.state.write().unwrap().kv;
            match kv.get(&key) {
                Some(val) if val == &from => {
                    kv.insert(key.clone(), to.clone());
                    Res::Ok
                }
                Some(_) => Res::Err22,
                None => Res::Err20,
            }
        };
        match res {
            Res::Ok => {
                self.reply(
                    src,
                    LinKv::CasOk {
                        in_reply_to: msg_id,
                        msg_id: Some(self.next_msg_id()),
                    },
                )
                .await?;
            }
            Res::Err20 => {
                self.reply_error(
                    src,
                    WError {
                        in_reply_to: msg_id,
                        code: 20,
                        text: "Key does not exist".to_string(),
                    },
                )
                .await?;
            }
            Res::Err22 => {
                self.reply_error(
                    src,
                    WError {
                        in_reply_to: msg_id,
                        code: 22,
                        text: "Value does not match".to_string(),
                    },
                )
                .await?;
            }
        }
        Ok(())
    }

    fn next_msg_id(&self) -> MsgId {
        self.msg_id.fetch_add(1, SeqCst)
    }

    async fn reply(&self, dest: Id, body: LinKv) -> Status {
        self.net
            .txq
            .send(Msg {
                src: self.id.clone(),
                dest,
                body: Body::Workload(body),
            })
            .await?;
        Ok(())
    }

    async fn reply_error(&self, dest: Id, body: WError) -> Status {
        self.net
            .txq
            .send(Msg {
                src: self.id.clone(),
                dest,
                body: Body::Error(body),
            })
            .await?;
        Ok(())
    }
}
