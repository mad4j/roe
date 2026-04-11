pub mod commands;
pub mod output;

pub mod proto {
    pub mod application_factory {
        tonic::include_proto!("hdds.application_factory");
    }

    pub mod deploy_manager {
        tonic::include_proto!("hdds.deploy_manager");
    }

    pub mod managed_application {
        tonic::include_proto!("hdds.managed_application");
    }

    pub mod configurable_application {
        tonic::include_proto!("hdds.configurable_application");
    }
}
