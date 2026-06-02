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

    pub(crate) const fn get_id(&self) -> ShaderID {
        self.id
    }

    pub(crate) fn dispose(&self) {
        let _ = self.sender.send(command::Command::DisposeShader(self.id));
    }
}
