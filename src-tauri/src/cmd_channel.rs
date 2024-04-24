use std::any::Any;
use tokio::sync::{mpsc, oneshot};

pub type Param = Box<dyn Any + Send>;

pub struct Cmd<ID> {
    pub id: ID,
    pub data: Option<Param>,
    resp_tx: oneshot::Sender<Option<Param>>,
}

pub fn channel<ID>() -> (CmdSender<ID>, CmdReceiver<ID>) {
    let (tx, rx) = mpsc::channel::<Cmd<ID>>(100);
    let sender = CmdSender {
        tx,
    };
    let receiver = CmdReceiver {
        rx,
    };
    (sender, receiver)
}

//Sender
pub struct CmdSender<ID> {
    tx: mpsc::Sender<Cmd<ID>>,
}

impl<ID> CmdSender<ID> {
    pub async fn send_io<A, B: 'static>(&self, id: ID, data: Option<A>) -> B
        where A: Any + Send
    {
        let (resp_tx, resp_rx) = oneshot::channel::<Option<Param>>();

        self.tx.try_send(Cmd {
            id,
            data: data.map(|x| {
                Box::new(x) as Box<dyn Any + Send>
            }),
            resp_tx,
        }).unwrap();

        resp_rx.await.unwrap().map(|x| {
            *x.downcast::<B>().unwrap()
        }).unwrap()
    }

    pub async fn send_o<B: 'static>(&self, id: ID) -> B {
        let (resp_tx, resp_rx) = oneshot::channel::<Option<Param>>();

        self.tx.try_send(Cmd {
            id,
            data: None,
            resp_tx,
        }).unwrap();

        resp_rx.await.unwrap().map(|x| {
           *x.downcast::<B>().unwrap()
        }).unwrap()
    }
}

//Receiver
pub struct CmdReceiver<ID> {
    rx: mpsc::Receiver<Cmd<ID>>,
}

impl<ID> CmdReceiver<ID> {
    pub fn get<F>(&mut self, handler: F) where
        F: FnOnce(ID, Option<Param>) -> Option<Param>
    {
        if let Ok(cmd) = self.rx.try_recv() {
            let resp = handler(cmd.id, cmd.data);
            cmd.resp_tx.send(resp).unwrap();
        }
    }
}