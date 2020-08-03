use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
	Start(Option<Box<[u8]>>),
	GetState,
	Close,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
	Restart,
	SetState(Option<Box<[u8]>>),
}
