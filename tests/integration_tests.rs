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
        .args(&["-o", "csv", "-j", ".users[].friends[]", "-K"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/friends.csv").unwrap());
}

#[test]
fn json_to_csv_collections_as_json() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.json")
        .args(&["-o", "csv", "-j", ".users[]", "-K"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/users.csv").unwrap());
}

#[test]
fn json_to_gron() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.json")
        .args(&["-o", "gron"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.js").unwrap());
}

#[test]
fn json_to_hcl() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.json")
        .args(&["-o", "hcl"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.hcl").unwrap());
}

#[test]
fn json_to_hcl_compact() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.json")
        .args(&["-o", "hcl", "--compact"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.compact.hcl").unwrap());
}

#[test]
fn hcl_to_json() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/math.hcl")
        .args(&["-o", "json"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/math.json").unwrap());
}

#[test]
fn hcl_to_json_simplified() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/math.hcl")
        .args(&["-o", "json", "--simplify"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/math.simplified.json").unwrap());
}

#[test]
fn gron_to_json() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.js")
        .args(&["-i", "gron", "-n", "-j", ".json"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.js.ungron.json").unwrap());
}

#[test]
fn encoding_required_for_stdin() {
    Command::cargo_bin("dts")
        .unwrap()
        .pipe_stdin("tests/fixtures/example.js")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unable to detect input encoding, please provide it explicitly via -i",
        ));
}

#[test]
fn encoding_inferred_from_first_line() {
    Command::cargo_bin("dts")
        .unwrap()
        .pipe_stdin("tests/fixtures/example.json")
        .unwrap()
        .assert()
        .success();
}

#[test]
fn multiple_sinks_require_array() {
    Command::cargo_bin("dts")
        .unwrap()
        .args(&["-i", "json", "-O", "-", "-O", "-"])
        .write_stdin("{}")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "when using multiple output files, the data must be an array",
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
fn merge_json() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.json")
        .args(&["-j", "reduce .users[] as $item ({}; . + $item)", "-n"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.merged.json").unwrap());
}

#[test]
fn filter_expression_from_file() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.json")
        .args(&["-j", "@tests/fixtures/filter.jq", "-n"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.filtered.json").unwrap());
}

#[test]
fn continue_on_error() {
    // Test for the failure first without the --continue-on-error flag to catch potential
    // regressions.
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.js")
        .arg("tests/fixtures/example.json")
        .args(&[
            "-i",
            "json",
            "-j",
            ".[] | reduce .users[] as $item ({}; . + $item)",
            "-n",
        ])
        .assert()
        .failure();

    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.js")
        .arg("tests/fixtures/example.json")
        .args(&[
            "-i",
            "json",
            "-j",
            ".[] | reduce .users[] as $item ({}; . + $item)",
            "-n",
            "--continue-on-error",
        ])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.merged.json").unwrap());
}

#[test]
fn yaml_to_multi_doc_yaml() {
    Command::cargo_bin("dts")
        .unwrap()
        .arg("tests/fixtures/example.yaml")
        .args(&["-o", "yaml", "--multi-doc-yaml", "-j", ".users[]"])
        .assert()
        .success()
        .stdout(read("tests/fixtures/example.multi-doc.yaml").unwrap());
}
