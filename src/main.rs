use tokio::sync::{Mutex, oneshot};
use tonic::{Request, Response, Status, transport::Server};

pub mod deploy_manager {
    tonic::include_proto!("deploy_manager");
}

pub mod managed_application {
    tonic::include_proto!("managed_application");
}

use deploy_manager::{
    DeployRequest, DeployResponse,
    deploy_manager_server::{DeployManager, DeployManagerServer},
};

use managed_application::{
    InfoRequest, InfoResponse, ListeningAddress,
    TerminateRequest, TerminateResponse,
    managed_application_server::{ManagedApplication, ManagedApplicationServer},
};

#[derive(Debug, Default)]
pub struct DeployManagerService;

#[tonic::async_trait]
impl DeployManager for DeployManagerService {
    async fn deploy(
        &self,
        request: Request<DeployRequest>,
    ) -> Result<Response<DeployResponse>, Status> {
        let req = request.into_inner();

        if req.yaml_content.is_empty() {
            return Err(Status::invalid_argument("yaml_content must not be empty"));
        }

        let env_summary: Vec<String> = req
            .env_vars
            .iter()
            .map(|e| format!("{}={}", e.key, e.value))
            .collect();

        let message = if env_summary.is_empty() {
            format!(
                "Deployment successful. YAML content length: {} bytes.",
                req.yaml_content.len()
            )
        } else {
            format!(
                "Deployment successful. YAML content length: {} bytes. Env vars: [{}].",
                req.yaml_content.len(),
                env_summary.join(", ")
            )
        };

        Ok(Response::new(DeployResponse {
            success: true,
            report: vec![message],
        }))
    }
}

#[derive(Debug)]
pub struct ManagedApplicationService {
    app_name: String,
    listening_addresses: Vec<ListeningAddress>,
    shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
}

#[tonic::async_trait]
impl ManagedApplication for ManagedApplicationService {
    async fn info(&self, _request: Request<InfoRequest>) -> Result<Response<InfoResponse>, Status> {
        Ok(Response::new(InfoResponse {
            app_name: self.app_name.clone(),
            listening_addresses: self.listening_addresses.clone(),
        }))
    }

    async fn terminate(
        &self,
        request: Request<TerminateRequest>,
    ) -> Result<Response<TerminateResponse>, Status> {
        let reason = request.into_inner().reason;
        let suffix = if reason.is_empty() {
            String::new()
        } else {
            format!(": {}", reason)
        };
        let message = format!("Termination accepted{}. Shutting down.", suffix);

        let mut guard = self.shutdown_tx.lock().await;
        if let Some(tx) = guard.take() {
            let _ = tx.send(());
        }

        Ok(Response::new(TerminateResponse {
            success: true,
            message,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const LISTEN_ADDR: &str = "[::1]:50051";

    let addr = LISTEN_ADDR.parse()?;
    let deploy_service = DeployManagerService;

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let managed_app_service = ManagedApplicationService {
        app_name: "roe".to_string(),
        listening_addresses: vec![ListeningAddress {
            address: LISTEN_ADDR.to_string(),
            services: vec![
                "deploy_manager.DeployManager".to_string(),
                "managed_application.ManagedApplication".to_string(),
            ],
        }],
        shutdown_tx: Mutex::new(Some(shutdown_tx)),
    };

    println!("DeployManager gRPC server listening on {addr}");

    Server::builder()
        .add_service(DeployManagerServer::new(deploy_service))
        .add_service(ManagedApplicationServer::new(managed_app_service))
        .serve_with_shutdown(addr, async {
            let _ = shutdown_rx.await;
        })
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use deploy_manager::EnvVar;
    use managed_application::TerminateRequest;

    #[tokio::test]
    async fn test_deploy_success() {
        let service = DeployManagerService::default();

        let request = Request::new(DeployRequest {
            yaml_content: "name: my-app\nversion: 1.0".to_string(),
            env_vars: vec![
                EnvVar {
                    key: "ENV".to_string(),
                    value: "production".to_string(),
                },
                EnvVar {
                    key: "PORT".to_string(),
                    value: "8080".to_string(),
                },
            ],
        });

        let response = service.deploy(request).await.unwrap();
        let body = response.into_inner();

        assert!(body.success);
        assert!(body.report[0].contains("Deployment successful"));
        assert!(body.report[0].contains("ENV=production"));
        assert!(body.report[0].contains("PORT=8080"));
    }

    #[tokio::test]
    async fn test_deploy_no_env_vars() {
        let service = DeployManagerService::default();

        let request = Request::new(DeployRequest {
            yaml_content: "name: my-app".to_string(),
            env_vars: vec![],
        });

        let response = service.deploy(request).await.unwrap();
        let body = response.into_inner();

        assert!(body.success);
        assert!(body.report[0].contains("Deployment successful"));
    }

    #[tokio::test]
    async fn test_deploy_empty_yaml_returns_error() {
        let service = DeployManagerService::default();

        let request = Request::new(DeployRequest {
            yaml_content: "".to_string(),
            env_vars: vec![],
        });

        let result = service.deploy(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_info_returns_app_name_and_addresses() {
        let service = ManagedApplicationService {
            app_name: "test-app".to_string(),
            listening_addresses: vec![ListeningAddress {
                address: "[::1]:50051".to_string(),
                services: vec!["my.Service".to_string()],
            }],
            shutdown_tx: Mutex::new(None),
        };

        let response = service.info(Request::new(InfoRequest {})).await.unwrap();
        let body = response.into_inner();

        assert_eq!(body.app_name, "test-app");
        assert_eq!(body.listening_addresses.len(), 1);
        assert_eq!(body.listening_addresses[0].address, "[::1]:50051");
        assert_eq!(body.listening_addresses[0].services, vec!["my.Service"]);
    }

    #[tokio::test]
    async fn test_info_no_addresses() {
        let service = ManagedApplicationService {
            app_name: "empty-app".to_string(),
            listening_addresses: vec![],
            shutdown_tx: Mutex::new(None),
        };

        let response = service.info(Request::new(InfoRequest {})).await.unwrap();
        let body = response.into_inner();

        assert_eq!(body.app_name, "empty-app");
        assert!(body.listening_addresses.is_empty());
    }

    #[tokio::test]
    async fn test_terminate_success() {
        let (tx, rx) = oneshot::channel::<()>();
        let service = ManagedApplicationService {
            app_name: "test-app".to_string(),
            listening_addresses: vec![],
            shutdown_tx: Mutex::new(Some(tx)),
        };

        let response = service
            .terminate(Request::new(TerminateRequest {
                reason: String::new(),
            }))
            .await
            .unwrap();
        let body = response.into_inner();

        assert!(body.success);
        assert!(body.message.contains("Termination accepted"));
        assert!(rx.await.is_ok());
    }

    #[tokio::test]
    async fn test_terminate_with_reason() {
        let (tx, rx) = oneshot::channel::<()>();
        let service = ManagedApplicationService {
            app_name: "test-app".to_string(),
            listening_addresses: vec![],
            shutdown_tx: Mutex::new(Some(tx)),
        };

        let response = service
            .terminate(Request::new(TerminateRequest {
                reason: "maintenance".to_string(),
            }))
            .await
            .unwrap();
        let body = response.into_inner();

        assert!(body.success);
        assert!(body.message.contains("maintenance"));
        assert!(rx.await.is_ok());
    }

    #[tokio::test]
    async fn test_terminate_idempotent() {
        let (tx, _rx) = oneshot::channel::<()>();
        let service = ManagedApplicationService {
            app_name: "test-app".to_string(),
            listening_addresses: vec![],
            shutdown_tx: Mutex::new(Some(tx)),
        };

        // First terminate succeeds
        let first = service
            .terminate(Request::new(TerminateRequest {
                reason: String::new(),
            }))
            .await
            .unwrap();
        assert!(first.into_inner().success);

        // Second terminate still succeeds (sender already consumed)
        let second = service
            .terminate(Request::new(TerminateRequest {
                reason: String::new(),
            }))
            .await
            .unwrap();
        assert!(second.into_inner().success);
    }
}
