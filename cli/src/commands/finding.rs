use std::{
    fs::{self, File},
    io::{self, BufRead},
    path::{Path, PathBuf},
    process::Command,
    string::String,
};

use crate::config::{BatConfig, BatConfigValidation};

pub fn create_finding_file(finding_name: String, informational: bool) {
    BatConfig::validate_bat_config();
    validate_config_create_finding_file(finding_name.clone());
    copy_template_to_findings_to_review(finding_name, informational)
}

pub fn prepare_all() {
    BatConfig::validate_bat_config();
    let to_review_path = BatConfig::get_auditor_findings_to_review_path(None);
    for to_review_file in fs::read_dir(to_review_path).unwrap() {
        let file = to_review_file.unwrap();
        let file_name = file.file_name();
        if file_name.to_str().unwrap() == ".gitkeep" {
            continue;
        }
        let mut file_name_tokenized = file_name
            .to_str()
            .unwrap()
            .to_string()
            .split("-")
            .map(|token| token.to_string())
            .collect::<Vec<String>>();
        let severity_flags = ["1", "2", "3", "4"];
        let finding_name = if severity_flags.contains(&file_name_tokenized[0].as_str()) {
            file_name_tokenized.remove(0);
            file_name_tokenized.join("-")
        } else {
            file_name_tokenized.join("-")
        };
        let open_file = File::open(file.path()).unwrap();
        let file_lines = io::BufReader::new(open_file).lines().map(|l| l.unwrap());
        for line in file_lines {
            if line.contains("Severity:") {
                let mut file_severity = line
                    .replace("**Severity:**", "")
                    .replace(" ", "")
                    .to_lowercase();
                let severity = match file_severity.as_str() {
                    "high" => "1",
                    "medium" => "2",
                    "low" => "3",
                    "informational" => "4",
                    &_ => panic!(
                        "severity: {:?} not recongnized in file {:?}",
                        file_severity,
                        file.path()
                    ),
                };
                Command::new("mv")
                    .args([
                        file.path(),
                        PathBuf::from(BatConfig::get_auditor_findings_to_review_path(Some(
                            severity.to_string() + "-" + finding_name.replace(".md", "").as_str(),
                        ))),
                    ])
                    .output()
                    .unwrap();
            }
        }
    }
    println!("All to-review findings severity tags updated")
}

// prepare_all

// create_finding_file
fn validate_config_create_finding_file(finding_name: String) {
    let findings_to_review_path = BatConfig::get_auditor_findings_to_review_path(None);
    // check auditor/findings/to_review folder exists
    if !Path::new(&findings_to_review_path).is_dir() {
        panic!("Folder not found: {:#?}", findings_to_review_path);
    }
    // check if file exists in to_review
    let finding_file_path = findings_to_review_path + &finding_name + ".md";
    if Path::new(&finding_file_path).is_file() {
        panic!("Finding file already exists: {:#?}", finding_file_path);
    }
}

fn copy_template_to_findings_to_review(finding_name: String, informational: bool) {
    let template_path = if informational {
        BatConfig::get_informational_template_path()
    } else {
        BatConfig::get_finding_template_path()
    };
    let new_file_path = BatConfig::get_auditor_findings_to_review_path(Some(finding_name.clone()));
    let output = Command::new("cp")
        .args([template_path, new_file_path.clone()])
        .output()
        .unwrap()
        .status
        .exit_ok();
    if let Err(output) = output {
        panic!("Finding creation failed with reason: {:#?}", output)
    };
    println!(
        "Finding file successfully created at: {:?}",
        new_file_path.clone()
    );
}