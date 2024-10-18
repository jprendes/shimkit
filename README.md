# shimkit

Library for writing containerd shims

# Getting Started

Create an executable file to print the path to the shim server
```bash
cat <<EOF | sudo tee /usr/local/bin/containerd-shim-logger-v1 > /dev/null
#!/bin/bash
cat /tmp/shimkit-logger-socket
EOF
sudo chmod a+x /usr/local/bin/containerd-shim-logger-v1
```

Build the logger example
```bash
cargo build -p shimkit --example logger
```

Then run the shim in a different terminal
```bash
sudo ./target/debug/examples/logger -id 123 daemon > /tmp/shimkit-logger-socket
```

Finally, run a container
```bash
docker run --runtime=io.containerd.logger.v1 hello-world
```

The `docker run` command will fail because the shim is a stub, but you will se the requests that `containerd` did on the `shim` printed to the terminal
