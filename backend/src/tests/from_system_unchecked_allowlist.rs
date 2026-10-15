use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const ALLOWED_ESCAPE_HATCH_FILES: [&str; 12] = [
    "src/ai_configuration/service.rs",
    "src/authorization/dto.rs",
    "src/demo/service.rs",
    "src/scheduler/mod.rs",
    "src/tenant/invitation/service.rs",
    "src/tenant/service.rs",
    "src/tests/attribution.rs",
    "src/tests/authorization_gates.rs",
    "src/tests/common/tenancy.rs",
    "src/tests/invitations.rs",
    "src/tests/platform.rs",
    "src/tests/startup.rs",
];

const THIS_FILE: &str = "src/tests/from_system_unchecked_allowlist.rs";

fn collect_rust_files(directory: &Path, collected: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rust_files(&path, collected);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            collected.push(path);
        }
    }
}

#[test]
fn tenant_id_escape_hatch_allowlist() {
    let mut rust_files = Vec::new();
    collect_rust_files(Path::new("src"), &mut rust_files);

    let mut files_using_escape_hatch = BTreeSet::new();
    for path in rust_files {
        let normalised_path = path.to_string_lossy().replace('\\', "/");
        if normalised_path == THIS_FILE {
            continue;
        }
        if std::fs::read_to_string(&path)
            .unwrap()
            .contains("from_system_unchecked(")
        {
            files_using_escape_hatch.insert(normalised_path);
        }
    }

    let allowed: BTreeSet<String> = ALLOWED_ESCAPE_HATCH_FILES
        .iter()
        .map(|value| value.to_string())
        .collect();

    assert_eq!(
        files_using_escape_hatch, allowed,
        "unreviewed use of from_system_unchecked. Every file constructing a TenantId or \
         UserId outside a verified request context must be listed in \
         ALLOWED_ESCAPE_HATCH_FILES and reviewed."
    );
}
