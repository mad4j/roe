# roe

**roe-cli** is a command-line gRPC client written in Rust for interacting with two services — `DeployManager` and `ManagedApplication`.

---

## Table of Contents

- [Services](#services)
  - [DeployManager](#deploymanager)
  - [ManagedApplication](#managedapplication)
- [Building](#building)
- [CLI interface](#cli-interface)
  - [Global options](#global-options)
  - [deploy](#deploy)
  - [info](#info)
  - [terminate](#terminate)

---

## Services

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

`roe-cli` is a thin gRPC client that wraps the two services.

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
