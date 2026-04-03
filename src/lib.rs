pub mod commands;
pub mod output;

pub mod proto {
    pub mod deploy_manager {
        tonic::include_proto!("deploy_manager");
    }

    pub mod managed_application {
        tonic::include_proto!("managed_application");
    }
}
