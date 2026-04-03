pub mod commands;
pub mod output;

pub mod proto {
    pub mod application_factory {
        tonic::include_proto!("application_factory");
    }

    pub mod deploy_manager {
        tonic::include_proto!("deploy_manager");
    }

    pub mod managed_application {
        tonic::include_proto!("managed_application");
    }
}
