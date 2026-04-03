# roe

**roe-cli** is a command-line gRPC client written in Rust for interacting with three services: `DeployManager`, `ManagedApplication`, and `ApplicationFactory`.

---

## Table of Contents

- [Services](#services)
  - [ApplicationFactory](#applicationfactory)
  - [DeployManager](#deploymanager)
  - [ManagedApplication](#managedapplication)
- [Building](#building)
- [CLI interface](#cli-interface)
  - [Global options](#global-options)
  - [application](#application)
  - [deploy](#deploy)
  - [info](#info)
  - [terminate](#terminate)

---

## Services

### ApplicationFactory

Defined in [`proto/application_factory.proto`](proto/application_factory.proto).

The `ApplicationFactory` service is intended as a higher-level orchestration layer built on top of the existing lower-level services:

- `ActivateApplication` can be implemented by delegating deployment logic to `DeployManager.Deploy`.
- `ListActiveApplications` can be implemented by returning the currently tracked active instances.
- `TerminateApplication` can be implemented by delegating graceful termination to the managed application lifecycle.

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `ActivateApplication` | `ActivateApplicationRequest` | `ActivateApplicationResponse` | Activates a new application instance from YAML configuration and environment variables. |
| `ListActiveApplications` | `ListActiveApplicationsRequest` | `ListActiveApplicationsResponse` | Returns all active application instances. |
| `TerminateApplication` | `TerminateApplicationRequest` | `TerminateApplicationResponse` | Terminates a specific active application instance. |

**`ActivateApplicationRequest`**

| Field | Type | Description |
|-------|------|-------------|
| `yaml_content` | `string` | Content of the YAML configuration file used for activation/deployment. |
| `env_vars` | `repeated deploy_manager.EnvVar` | Environment variables to apply during activation/deployment. |

**`ActivateApplicationResponse`**

| Field | Type | Description |
|-------|------|-------------|
| `success` | `bool` | `true` when activation was accepted. |
| `application_id` | `string` | Identifier assigned to the activated application instance. |
| `report` | `repeated string` | Human-readable lines describing activation results. |

**`ListActiveApplicationsResponse`**

| Field | Type | Description |
|-------|------|-------------|
| `applications` | `repeated ActiveApplication` | Collection of active application instances. |

**`ActiveApplication`**

| Field | Type | Description |
|-------|------|-------------|
| `application_id` | `string` | Identifier of the active application instance. |
| `app_name` | `string` | Human-readable application name. |

**`TerminateApplicationRequest`**

| Field | Type | Description |
|-------|------|-------------|
| `application_id` | `string` | Identifier of the active application instance to terminate. |
| `reason` | `string` | Optional human-readable reason for termination. |

**`TerminateApplicationResponse`**

| Field | Type | Description |
|-------|------|-------------|
| `success` | `bool` | `true` when termination was accepted. |
| `message` | `string` | Human-readable message describing the result. |

---

### DeployManager

Defined in [`proto/deploy_manager.proto`](proto/deploy_manager.proto).

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `Deploy` | `DeployRequest` | `DeployResponse` | Accepts a YAML configuration and an optional list of environment variables, and returns a deployment report. |

**`DeployRequest`**

| Field | Type | Description |
|-------|------|-------------|
| `yaml_content` | `string` | Content of the YAML configuration file (required, must not be empty). |
| `env_vars` | `repeated EnvVar` | Environment variables to apply during deployment (`key`/`value` pairs). |

**`DeployResponse`**

| Field | Type | Description |
|-------|------|-------------|
| `success` | `bool` | `true` when the deployment was accepted. |
| `report` | `repeated string` | Human-readable lines describing the result. |

---

### ManagedApplication

Defined in [`proto/managed_application.proto`](proto/managed_application.proto).

| RPC | Request | Response | Description |
|-----|---------|----------|-------------|
| `Info` | `InfoRequest` | `InfoResponse` | Returns the application name and the list of addresses/services it is listening on. |
| `Terminate` | `TerminateRequest` | `TerminateResponse` | Requests a graceful shutdown of the server. |

**`InfoResponse`**

| Field | Type | Description |
|-------|------|-------------|
| `app_name` | `string` | Human-readable name of the application. |
| `listening_addresses` | `repeated ListeningAddress` | Each entry contains an `address` and the `services` reachable at that address. |

**`TerminateRequest`**

| Field | Type | Description |
|-------|------|-------------|
| `reason` | `string` | Optional human-readable reason for the shutdown request. |

**`TerminateResponse`**

| Field | Type | Description |
|-------|------|-------------|
| `success` | `bool` | `true` when the termination was accepted. |
| `message` | `string` | Human-readable confirmation message. |

---

## Building

```bash
cargo build --release
```

The build step also compiles the `.proto` files via `tonic-build` (see `build.rs`).

---

## CLI interface

`roe-cli` is a thin gRPC client that wraps the three services.

```bash
cargo run --bin roe-cli -- [OPTIONS] <COMMAND>
# or after release build:
./target/release/roe-cli [OPTIONS] <COMMAND>
```

### Global options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--address <URL>` | `-a` | `http://[::1]:50051` | gRPC server address. |
| `--output <FORMAT>` | `-o` | `table` | Output format: `table` or `json`. |

### application

Calls RPCs on the `ApplicationFactory` service.

#### application activate

Calls the `ActivateApplication` RPC.

```bash
roe-cli application activate [--yaml-content <YAML>] [--env-var <KEY=VALUE>]...
roe-cli application activate --json '<JSON>'
```

| Flag | Description |
|------|-------------|
| `--yaml-content <YAML>` | YAML configuration string (required unless `--json` is used). |
| `--env-var <KEY=VALUE>` | Environment variable in `KEY=VALUE` format. Repeatable. |
| `--json <JSON>` | Provide the full request as a JSON object (mutually exclusive with `--yaml-content` / `--env-var`). |

**Examples**

```bash
roe-cli application activate --yaml-content "name: my-app" --env-var ENV=production

roe-cli application activate --json '{"yaml_content":"name: my-app","env_vars":[{"key":"ENV","value":"production"}]}'
```

#### application list

Calls the `ListActiveApplications` RPC.

```bash
roe-cli application list
```

#### application terminate

Calls the `TerminateApplication` RPC.

```bash
roe-cli application terminate --application-id <ID> [--reason <TEXT>]
roe-cli application terminate --json '<JSON>'
```

| Flag | Description |
|------|-------------|
| `--application-id <ID>` | Active application identifier (required unless `--json` is used). |
| `--reason <TEXT>` | Optional reason sent to the server for the termination request. |
| `--json <JSON>` | Provide the full request as a JSON object (mutually exclusive with `--application-id` / `--reason`). |

**Examples**

```bash
roe-cli application terminate --application-id app-123 --reason "maintenance window"

roe-cli application terminate --json '{"application_id":"app-123","reason":"maintenance window"}'
```

### deploy

Calls the `Deploy` RPC on the `DeployManager` service.

```
roe-cli deploy [--yaml-content <YAML>] [--env-var <KEY=VALUE>]...
roe-cli deploy --json '<JSON>'
```

| Flag | Description |
|------|-------------|
| `--yaml-content <YAML>` | YAML configuration string (required unless `--json` is used). |
| `--env-var <KEY=VALUE>` | Environment variable in `KEY=VALUE` format. Repeatable. |
| `--json <JSON>` | Provide the full request as a JSON object (mutually exclusive with `--yaml-content` / `--env-var`). |

**Examples**

```bash
# Using individual flags
roe-cli deploy --yaml-content "name: my-app" --env-var ENV=production --env-var PORT=8080

# Using JSON input
roe-cli deploy --json '{"yaml_content":"name: my-app","env_vars":[{"key":"ENV","value":"production"}]}'

# JSON output format
roe-cli -o json deploy --yaml-content "name: my-app"
```

**Table output (default)**

```
+---------+--------------------------------------------------------------+
| Success | Report                                                       |
+---------+--------------------------------------------------------------+
| true    | Deployment successful. YAML content length: 14 bytes.        |
+---------+--------------------------------------------------------------+
```

**JSON output (`-o json`)**

```json
{
  "success": true,
  "report": [
    "Deployment successful. YAML content length: 14 bytes."
  ]
}
```

---

### info

Calls the `Info` RPC on the `ManagedApplication` service and prints the application name together with the addresses and services it is listening on.

```
roe-cli info
```

**Table output (default)**

```
Application: roe
+---------------+----------------------------------------------------------+
| Address       | Services                                                 |
+---------------+----------------------------------------------------------+
| [::1]:50051   | deploy_manager.DeployManager, managed_application.ManagedApplication |
+---------------+----------------------------------------------------------+
```

**JSON output (`-o json`)**

```json
{
  "app_name": "roe",
  "listening_addresses": [
    {
      "address": "[::1]:50051",
      "services": [
        "deploy_manager.DeployManager",
        "managed_application.ManagedApplication"
      ]
    }
  ]
}
```

---

### terminate

Calls the `Terminate` RPC on the `ManagedApplication` service.

```
roe-cli terminate [--reason <TEXT>]
```

| Flag | Description |
|------|-------------|
| `--reason <TEXT>` | Optional reason sent to the server for the graceful shutdown request. |

**Examples**

```bash
# Request graceful shutdown with no reason
roe-cli terminate

# Request graceful shutdown with a reason
roe-cli terminate --reason "maintenance window"

# JSON output format
roe-cli -o json terminate --reason "deploy completed"
```

**Table output (default)**

```
+---------+------------------------------+
| Success | Message                      |
+---------+------------------------------+
| true    | Termination accepted         |
+---------+------------------------------+
```

**JSON output (`-o json`)**

```json
{
  "success": true,
  "message": "Termination accepted"
}
```
