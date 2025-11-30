use tokio::sync::mpsc;

use super::{ShaderID, command};

pub(crate) struct Shader {
    id: ShaderID,
    sender: mpsc::UnboundedSender<command::Command>,
}

impl Shader {
    pub(super) fn new(id: ShaderID, sender: mpsc::UnboundedSender<command::Command>) -> Self {
        Self { id, sender }
    }

    pub(super) const fn get_id(&self) -> ShaderID {
        self.id
    }
}
