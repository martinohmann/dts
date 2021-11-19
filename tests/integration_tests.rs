use assert_cmd::Command;
use predicates::prelude::*;
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
            "--transform",
            "j=..friends,f",
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
        .args(&["-i", "json", "-o", "csv", "-t", "jsonpath=.users[*]", "-K"])
        .pipe_stdin("tests/fixtures/example.json")
        .unwrap()
        .assert()
        .success()
        .code(0)
        .stdout(read("tests/fixtures/users.csv").unwrap());
}

#[test]
fn json_to_gron() {
    Command::cargo_bin("dts")
        .unwrap()
        .args(&["-i", "json", "-o", "gron"])
        .pipe_stdin("tests/fixtures/example.json")
        .unwrap()
        .assert()
        .success()
        .code(0)
        .stdout(read("tests/fixtures/example.js").unwrap());
}

#[test]
fn glob_required_for_dirs() {
    Command::cargo_bin("dts")
        .unwrap()
        .args(&["-i", "json", "tests/"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--glob is required if sources contain directories",
        ));
}

#[test]
fn deep_merge_json() {
    Command::cargo_bin("dts")
        .unwrap()
        .args(&["-i", "json", "-p", "-t", "j=.users,f,m", "-n"])
        .pipe_stdin("tests/fixtures/example.json")
        .unwrap()
        .assert()
        .success()
        .code(0)
        .stdout(read("tests/fixtures/example.merged.json").unwrap());
}
