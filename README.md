# shimkit

Library for writing containerd shims

## Task API

### Setup

1. Build the logger example
    ```bash
    cargo build --example logger
    ```

1. Create an executable script to print the path to the shim server
    ```bash
    cat <<EOF | sudo tee /usr/local/bin/containerd-shim-logger-v1 > /dev/null
    #!/bin/bash
    echo unix:///run/containerd/containerd-shim-logger-debug.sock
    EOF
    sudo chmod a+x /usr/local/bin/containerd-shim-logger-v1
    ```

3. Then run the shim
    ```bash
    sudo ./target/debug/examples/logger start
    ```

### Start a container

Now in a different terminal start a container with `docker run`
```bash
docker run --runtime=io.containerd.logger.v1 hello-world
```

The command will fail because the logger shim is just a stub, but you will see the requests that containerd did on the shim printed to the terminal.

## Sandbox API

### Setup

1. If you haven't, follow the setup in steps in [Task API](#task-api)
2. Enable containerd's sandbox API by setting the environent variable `ENABLE_CRI_SANDBOXES=sandboxed` when launching containerd. If you use systemd edit `/usr/lib/systemd/system/containerd.service` and in the `[Service]` section add `Environment=ENABLE_CRI_SANDBOXES=sandboxed`.
    ```ini
    ...
    [Service]
    ExecStartPre=-/user/bin/modprobe overlay
    ExecStart=/usr/bin/containerd
    Environment=ENABLE_CRI_SANDBOXES=sandboxed
    ...
    ```
3. Add the runtime to containerd's `config.toml` file `/etc/containerd/config.toml`. If the file doesn't exist, create it. A minimal example below:
    ```toml
    version = 2
    [plugins."io.containerd.grpc.v1.cri".containerd.runtimes.logger]
        runtime_type = "io.containerd.logger.v1"
        sandbox_mode = "shim"
    [plugins."io.containerd.grpc.v1.cri".containerd.runtimes.runc]
        runtime_type = "io.containerd.runc.v2"
    ```
4. Restart containerd. If you use systemd
    ```bash
    sudo systemctl daemon-reload
    sudo systemctl restart containerd
    ```

### Start a sandbox

Now start a new sandbox using `crictl runp`
```bash
cat <<EOF > /tmp/pod-config.yaml
metadata:
  name: my-sandbox
  namespace: default
  uid: abc123
EOF
sudo crictl \
    --runtime-endpoint=unix:///run/containerd/containerd.sock runp \
    --runtime=logger \
    /tmp/pod-config.yaml
```

The command will fail because the logger shim is just a stub, but you will see the requests that containerd did on the shim printed to the terminal.
