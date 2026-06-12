use std::{env::current_dir, fs::canonicalize, process::Command};

use insta::glob;
use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};

#[test]
fn test_compiler() {
    glob!("input/*.son", |path| {
        assert_cmd_snapshot!(
            Command::new(get_cargo_bin("spring")).arg(
                path.strip_prefix(canonicalize(current_dir().unwrap()).unwrap())
                    .unwrap()
                    .to_string_lossy()
                    .replace("\\", "/")
            )
        );
    });
}
