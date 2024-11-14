use std::ffi::OsStr;

use oci_spec::runtime::Spec;

const GROUP_LABELS: [&str; 2] = [
    "io.kubernetes.cri.sandbox-id",
    "io.containerd.runc.v2.group",
];

pub fn cri_sandbox_id() -> Option<String> {
    if let Ok(spec) = Spec::load("config.json") {
        if let Some(annotations) = spec.annotations() {
            for &label in GROUP_LABELS.iter() {
                if let Some(value) = annotations.get(label) {
                    return Some(value.clone());
                }
            }
        }
    }
    None
}

pub(crate) trait ToLossyString {
    fn to_lossy_string(&self) -> String;
}

impl<T: AsRef<OsStr>> ToLossyString for T {
    fn to_lossy_string(&self) -> String {
        self.as_ref().to_string_lossy().into_owned()
    }
}
