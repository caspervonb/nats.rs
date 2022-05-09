#[derive(Clone, Debug)]
pub struct Client {
    sender: mpsc::Sender<Command>,
}

impl Client {
    pub(crate) new(sender: mscp::Sender<Command>) -> Client {
        Client {
            sender,
        }
    }
}
