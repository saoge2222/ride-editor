use crate::rpc::message::{ResponseError, ERROR_INVALID_REQUEST, ERROR_SERVER_NOT_INITIALIZED};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerState {
    Uninitialized,
    Initializing,
    Initialized,
    Shutdown,
    Exited,
}

impl ServerState {
    pub fn validate_request(&self, method: &str) -> Result<(), ResponseError> {
        match self {
            ServerState::Uninitialized => {
                if method == "initialize" {
                    Ok(())
                } else {
                    Err(ResponseError {
                        code: ERROR_SERVER_NOT_INITIALIZED,
                        message: "服务器尚未初始化".to_string(),
                    })
                }
            }
            ServerState::Initializing => Err(ResponseError {
                code: ERROR_INVALID_REQUEST,
                message: "初始化进行中不允许请求".to_string(),
            }),
            ServerState::Initialized => Ok(()),
            ServerState::Shutdown => Err(ResponseError {
                code: ERROR_INVALID_REQUEST,
                message: "服务器已关闭".to_string(),
            }),
            ServerState::Exited => Err(ResponseError {
                code: ERROR_INVALID_REQUEST,
                message: "服务器已退出".to_string(),
            }),
        }
    }

    pub fn validate_notification(&self, method: &str) -> Result<(), ResponseError> {
        match self {
            ServerState::Uninitialized => {
                if method == "exit" {
                    Ok(())
                } else {
                    Err(ResponseError {
                        code: ERROR_SERVER_NOT_INITIALIZED,
                        message: "服务器尚未初始化".to_string(),
                    })
                }
            }
            ServerState::Initializing => {
                if method == "initialized" {
                    Ok(())
                } else {
                    Err(ResponseError {
                        code: ERROR_INVALID_REQUEST,
                        message: "初始化进行中只接受 initialized".to_string(),
                    })
                }
            }
            ServerState::Initialized => Ok(()),
            ServerState::Shutdown => {
                if method == "exit" {
                    Ok(())
                } else {
                    Err(ResponseError {
                        code: ERROR_INVALID_REQUEST,
                        message: "服务器已关闭，只接受 exit".to_string(),
                    })
                }
            }
            ServerState::Exited => Err(ResponseError {
                code: ERROR_INVALID_REQUEST,
                message: "服务器已退出".to_string(),
            }),
        }
    }

    pub fn transition(&mut self, next: ServerState) -> Result<(), ResponseError> {
        let valid = matches!(
            (*self, next),
            (ServerState::Uninitialized, ServerState::Initializing)
                | (ServerState::Initializing, ServerState::Initialized)
                | (ServerState::Initialized, ServerState::Shutdown)
                | (ServerState::Shutdown, ServerState::Exited)
        );
        if valid {
            *self = next;
            Ok(())
        } else {
            Err(ResponseError {
                code: ERROR_INVALID_REQUEST,
                message: "非法的生命周期迁移".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uninitialized_only_accepts_initialize() {
        let state = ServerState::Uninitialized;
        assert!(state.validate_request("initialize").is_ok());
        assert!(state.validate_request("shutdown").is_err());
        assert!(state.validate_notification("exit").is_ok());
        assert!(state.validate_notification("initialized").is_err());
    }

    #[test]
    fn initializing_only_accepts_initialized() {
        let state = ServerState::Initializing;
        assert!(state.validate_request("initialize").is_err());
        assert!(state.validate_notification("initialized").is_ok());
        assert!(state.validate_notification("exit").is_err());
    }

    #[test]
    fn initialized_accepts_everything() {
        let state = ServerState::Initialized;
        assert!(state.validate_request("shutdown").is_ok());
        assert!(state.validate_request("textDocument/completion").is_ok());
        assert!(state.validate_notification("exit").is_ok());
        assert!(state.validate_notification("textDocument/didOpen").is_ok());
    }

    #[test]
    fn shutdown_only_accepts_exit_notification() {
        let state = ServerState::Shutdown;
        assert!(state.validate_request("initialize").is_err());
        assert!(state.validate_notification("exit").is_ok());
        assert!(state.validate_notification("textDocument/didChange").is_err());
    }

    #[test]
    fn valid_transitions() {
        let mut state = ServerState::Uninitialized;
        assert!(state.transition(ServerState::Initializing).is_ok());
        assert!(state.transition(ServerState::Initialized).is_ok());
        assert!(state.transition(ServerState::Shutdown).is_ok());
        assert!(state.transition(ServerState::Exited).is_ok());
    }

    #[test]
    fn invalid_transition_rejected() {
        let mut state = ServerState::Uninitialized;
        assert!(state.transition(ServerState::Shutdown).is_err());
        assert_eq!(state, ServerState::Uninitialized);
    }

    #[test]
    fn repeat_initialize_rejected() {
        let mut state = ServerState::Initialized;
        assert!(state.transition(ServerState::Initializing).is_err());
        assert_eq!(state, ServerState::Initialized);
    }
}
