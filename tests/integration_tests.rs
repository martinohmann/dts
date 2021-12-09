use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::read_to_string as read;

#[test]
fn json_to_yaml() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.json")
        .args(&["-o", "yaml"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.yaml").unwrap());
}

#[test]
fn json_to_yaml_stdin() {
    Command::cargo_bin("dts")
        .unwrap()
        .args(&["-i", "json", "-o", "yaml"])
        .pipe_stdin("tests/fixtures/example.json")
        .unwrap()
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.yaml").unwrap());
}

#[test]
fn yaml_to_pretty_json() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.yaml")
        .args(&["-o", "json", "-n"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.json").unwrap());
}

#[test]
fn json_to_toml() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.json")
        .args(&["-o", "toml", "-c"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.toml").unwrap());
}

#[test]
fn json_to_csv_filtered_flattened_with_keys() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.json")
        .args(&["-o", "csv", "-t", "j=..friends,f", "-K"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/friends.csv").unwrap());
}

#[test]
fn json_to_csv_collections_as_json() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.json")
        .args(&["-o", "csv", "-t", "jsonpath=.users[*]", "-K"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/users.csv").unwrap());
}

#[test]
fn json_to_gron() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.json")
        .args(&["-tF=json", "-o", "gron"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.js").unwrap());
}

#[test]
fn gron_to_json() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.js")
        .args(&["-i", "gron"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.js.ungron.json").unwrap());
}

#[test]
fn encoding_required_for_stdin() {
    Command::cargo_bin("dts")
        .unwrap()
        .pipe_stdin("tests/fixtures/example.json")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Unable to detect input encoding, please provide it explicitly via -i",
        ));
}

#[test]
fn multiple_sinks_require_array() {
    Command::cargo_bin("dts")
        .unwrap()
        .args(&["-i", "json", "-O", "-", "-"])
        .write_stdin("{}")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "When using multiple output files, the data must be an array",
        ));
}

#[test]
fn glob_required_for_dirs() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/")
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
        .arg("tests/fixtures/example.json")
        .args(&["-t", "j=.users,f,m", "-n"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.merged.json").unwrap());
}
