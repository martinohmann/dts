use assert_cmd::Command;
use std::fs::read_to_string as read;

#[test]
fn json_to_yaml() {
    Command::cargo_bin("dts")
        .unwrap()
        .args(&["-i", "json", "-o", "yaml"])
        .pipe_stdin("tests/fixtures/example.json")
        .unwrap()
        .assert()
        .success()
        .code(0)
        .stdout(read("tests/fixtures/example.yaml").unwrap());
}

#[test]
fn yaml_to_pretty_json() {
    Command::cargo_bin("dts")
        .unwrap()
        .args(&["-i", "yaml", "-o", "json", "-p", "-n"])
        .pipe_stdin("tests/fixtures/example.yaml")
        .unwrap()
        .assert()
        .success()
        .code(0)
        .stdout(read("tests/fixtures/example.json").unwrap());
}

#[test]
fn json_to_toml() {
    Command::cargo_bin("dts")
        .unwrap()
        .args(&["-i", "json", "-o", "toml"])
        .pipe_stdin("tests/fixtures/example.json")
        .unwrap()
        .assert()
        .success()
        .code(0)
        .stdout(read("tests/fixtures/example.toml").unwrap());
}

#[test]
fn json_to_csv_filtered_flattened_with_keys() {
    Command::cargo_bin("dts")
        .unwrap()
        .args(&[
            "-i",
            "json",
            "-o",
            "csv",
            "-j",
            "..friends",
            "--flatten-arrays",
            "--keys-as-csv-headers",
        ])
        .pipe_stdin("tests/fixtures/example.json")
        .unwrap()
        .assert()
        .success()
        .code(0)
        .stdout(read("tests/fixtures/friends.csv").unwrap());
}

#[test]
fn json_to_csv_collections_as_json() {
    Command::cargo_bin("dts")
        .unwrap()
        .args(&[
            "-i",
            "json",
            "-o",
            "csv",
            "-j",
            ".users[*]",
            "--keys-as-csv-headers",
        ])
        .pipe_stdin("tests/fixtures/example.json")
        .unwrap()
        .assert()
        .success()
        .code(0)
        .stdout(read("tests/fixtures/users.csv").unwrap());
}
