use tonic::{transport::Server, Request, Response, Status};

pub mod deploy_manager {
    tonic::include_proto!("deploy_manager");
}

use deploy_manager::{
    deploy_manager_server::{DeployManager, DeployManagerServer},
    DeployRequest, DeployResponse,
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
            message,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = DeployManagerService;

    println!("DeployManager gRPC server listening on {addr}");

    Server::builder()
        .add_service(DeployManagerServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use deploy_manager::EnvVar;

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
        assert!(body.message.contains("Deployment successful"));
        assert!(body.message.contains("ENV=production"));
        assert!(body.message.contains("PORT=8080"));
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
        assert!(body.message.contains("Deployment successful"));
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
}
