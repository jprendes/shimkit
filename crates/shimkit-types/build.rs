use trapeze_codegen::Config;

fn main() {
    Config::new()
        .enable_type_names()
        .include_file("mod.rs")
        .compile_protos(
            &[
                "protos/gogoproto/gogo.proto",
                "protos/github.com/containerd/containerd/protobuf/plugin/fieldpath.proto",
                "protos/github.com/containerd/containerd/api/types/mount.proto",
                "protos/github.com/containerd/containerd/api/types/task/task.proto",
                "protos/github.com/containerd/cgroups/stats/v1/metrics.proto",
                "protos/microsoft/hcsshim/cmd/containerd-shim-runhcs-v1/stats/stats.proto",
                "protos/github.com/containerd/containerd/api/events/container.proto",
                "protos/github.com/containerd/containerd/api/events/content.proto",
                "protos/github.com/containerd/containerd/api/events/image.proto",
                "protos/github.com/containerd/containerd/api/events/namespace.proto",
                "protos/github.com/containerd/containerd/api/events/sandbox.proto",
                "protos/github.com/containerd/containerd/api/events/snapshot.proto",
                "protos/github.com/containerd/containerd/api/events/task.proto",
                "protos/github.com/containerd/containerd/runtime/v2/runc/options/oci.proto",
                "protos/github.com/containerd/containerd/api/runtime/task/v2/shim.proto",
                "protos/github.com/containerd/containerd/api/services/ttrpc/events/v1/events.proto",
                "protos/github.com/containerd/containerd/api/types/platform.proto",
                "protos/github.com/containerd/containerd/api/runtime/sandbox/v1/sandbox.proto",
                "protos/k8s.io/cri-api/pkg/apis/runtime/v1/api.proto",
            ],
            &["protos/"],
        )
        .expect("Failed to generate protos");
}

/*

*/
