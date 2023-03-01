use clap_verbosity_flag::LogLevel;
use error_stack::{FutureExt, IntoReport, Result, ResultExt};
use log::Level;

use serde_json::Map;
use serde_json::{json, Value};

use crate::batbelt::path::BatFile;
use crate::batbelt::templates::{TemplateError, TemplateResult};

use crate::BatCommands;

pub struct PackageJsonTemplate;

impl PackageJsonTemplate {
    pub fn create_package_with_init_script() -> Result<(), TemplateError> {
        let (script_key, script_value) = if cfg!(debug_assertions) {
            ("cargo::run::init", "cargo run init")
        } else {
            ("bat-cli::init", "bat-cli init")
        };
        let package_json = json!({
        "name": "bat_project",
        "version": "1.0.0",
        "description": "Bat project",
        "main": "index.js",
        "scripts":{
              script_key: script_value
            },
        "author": "",
        "license": "ISC"
        });

        let content = serde_json::to_string_pretty(&package_json)
            .into_report()
            .change_context(TemplateError)?;

        BatFile::PackageJson {
            to_create_project: true,
        }
        .write_content(false, &content)
        .change_context(TemplateError)?;
        Ok(())
    }

    pub fn create_package_json(log_level: Option<Level>) -> Result<(), TemplateError> {
        BatFile::PackageJson {
            to_create_project: false,
        }
        .write_content(false, &Self::get_package_json_content(log_level)?)
        .change_context(TemplateError)?;
        Ok(())
    }

    fn get_package_json_content(log_level: Option<Level>) -> TemplateResult<String> {
        let scripts_value = Self::get_scripts_serde_value(log_level)?;
        let package_json = json!({
        "name": "bat_project",
        "version": "1.0.0",
        "description": "Bat project",
        "main": "index.js",
        "scripts": scripts_value,
        "author": "",
        "license": "ISC"
        });

        Ok(serde_json::to_string_pretty(&package_json).unwrap())
    }

    fn get_scripts_serde_value(log_level: Option<Level>) -> TemplateResult<Value> {
        let (verbosity_flag, verbosity_level_name) = if let Some(level) = log_level {
            match level {
                Level::Warn => ("v".to_string(), level.to_string()),
                Level::Info => ("vv".to_string(), level.to_string()),
                Level::Debug => ("vvv".to_string(), level.to_string()),
                Level::Trace => ("vvvv".to_string(), level.to_string()),
                _ => ("".to_string(), "".to_string()),
            }
        } else {
            ("".to_string(), "".to_string())
        };
        let (script_key_prefix, script_value_prefix) = if cfg!(debug_assertions) {
            if verbosity_flag.is_empty() {
                (format!("cargo::run"), format!("cargo run",))
            } else {
                (
                    format!("cargo::run::{}", verbosity_level_name),
                    format!("cargo run -- -{}", verbosity_flag),
                )
            }
        } else {
            if verbosity_flag.is_empty() {
                (format!("bat-cli"), format!("bat-cli"))
            } else {
                (
                    format!("bat-cli::{}", verbosity_level_name),
                    format!("bat-cli -{}", verbosity_flag),
                )
            }
        };
        let kebab_commands_vec = BatCommands::get_kebab_commands();
        let mut scripts_map = Map::new();
        for kebab_comand in kebab_commands_vec {
            let (kebab_options_vec, kebab_command_name) = kebab_comand;
            if kebab_options_vec.is_empty() {
                let script_key = format!("{}::{}", script_key_prefix, kebab_command_name);
                let script_value = format!("{} {}", script_value_prefix, kebab_command_name);
                scripts_map.insert(script_key, script_value.into());
                continue;
            }
            for kebab_option in kebab_options_vec {
                let script_key = format!(
                    "{}::{}::{}",
                    script_key_prefix,
                    kebab_command_name,
                    kebab_option.clone()
                );
                let script_value = format!(
                    "{} {} {}",
                    script_value_prefix,
                    kebab_command_name,
                    kebab_option.clone()
                );
                scripts_map.insert(script_key, script_value.into());
            }
        }
        let serde_value: Value = scripts_map.into();
        Ok(serde_value)
    }
}

#[cfg(test)]
mod template_test {
    use crate::batbelt::templates::package_json_template::PackageJsonTemplate;

    #[test]
    fn test_get_package_json_content() {
        let json_content = PackageJsonTemplate::get_package_json_content(None).unwrap();
        println!("{}", json_content);
    }

    #[test]
    fn test_update_package_json_content() {
        // let co_vec = CodeOverhaulCommand::get_type_vec();
        // let findings_vec = FindingCommand::get_type_vec();
        // let repo_vec = RepositoryCommand::get_type_vec();
        // let miro_vec = MiroCommand::get_type_vec();
        // let sonar_vec = SonarCommand::get_type_vec();
        // let uno = 1;
        // let dos = 2;
        // let mut map = Map::new();
        //
        // let formatted = repo_vec
        //     .clone()
        //     .into_iter()
        //     .fold(vec![], |mut result, co_command| {
        //         let param = vec![
        //             format!("cargo::run::co::{}", co_command.to_string().to_kebab_case()),
        //             format!("cargo run co {}", co_command.to_string().to_kebab_case()),
        //         ];
        //         result.push(param.clone());
        //         map.insert(param[0].clone(), param[1].clone().into());
        //         // let json = json!({
        //         //     "scripts":{
        //         //         param[0].clone():param[1].clone()
        //         //     }
        //         // });
        //         // let string = serde_json::to_string_pretty(&json).unwrap();
        //         // println!("{string}");
        //         result
        //     });
        // let parse = formatted
        //     .into_iter()
        //     .map(|res| format!("\"{}\": \"{}\"", res[0], res[1]))
        //     .collect::<Vec<_>>()
        //     .join(", \n");
        // let value: Value = map.into();
        // let json = json!({ "scripts": value });
        // let string = serde_json::to_string(&json).unwrap();
        // fs::write("./package_test.json", &string).unwrap();
        // println!("{string}");

        // let serde: String =
        //     serde_json::from_str(&format!(r#"{{ script:{} }}"#, r#""hola":"chao""#)).unwrap();
        // // println!("{:#?}", internal);
        // // println!("{}", json);
        // println!("{}", serde);
    }
}
