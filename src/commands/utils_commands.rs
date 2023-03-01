use crate::batbelt;
use crate::batbelt::bat_dialoguer::BatDialoguer;
use crate::batbelt::command_line::{execute_command, CodeEditor};
use crate::batbelt::git::{deprecated_check_correct_branch, GitCommit};
use crate::batbelt::parser::entrypoint_parser::EntrypointParser;
use crate::batbelt::path::{BatFile, BatFolder};
use crate::batbelt::templates::code_overhaul_template::{
    CodeOverhaulTemplate, CoderOverhaulTemplatePlaceholders,
};
use crate::batbelt::BatEnumerator;
use crate::commands::{BatCommandEnumerator, CommandError, CommandResult};
use crate::config::{BatAuditorConfig, BatConfig};
use clap::Subcommand;
use colored::Colorize;
use error_stack::{Report, ResultExt};

use crate::batbelt::metadata::functions_metadata::FunctionMetadata;
use crate::batbelt::metadata::structs_metadata::StructMetadata;
use crate::batbelt::metadata::traits_metadata::TraitMetadata;
use crate::batbelt::metadata::{BatMetadataParser, BatMetadataType};
use crate::batbelt::parser::parse_formatted_path;
use crate::batbelt::templates::package_json_template::PackageJsonTemplate;
use log::Level;
use std::fs;

#[derive(
    Subcommand, Debug, strum_macros::Display, PartialEq, Clone, strum_macros::EnumIter, Default,
)]
pub enum ToolsCommands {
    /// Opens a file from metadata to code editor. If code editor is None, then prints the path
    #[default]
    Open,
    /// Customize the package.json according to certain log level
    PackageJson,
}

impl BatEnumerator for ToolsCommands {}

impl BatCommandEnumerator for ToolsCommands {
    fn execute_command(&self) -> CommandResult<()> {
        match self {
            ToolsCommands::Open => self.execute_open(),
            ToolsCommands::PackageJson => self.execute_package_json(),
        }
    }

    fn check_metadata_is_initialized(&self) -> bool {
        match self {
            ToolsCommands::Open => true,
            ToolsCommands::PackageJson => false,
        }
    }

    fn check_correct_branch(&self) -> bool {
        match self {
            ToolsCommands::Open => false,
            ToolsCommands::PackageJson => false,
        }
    }
}

impl ToolsCommands {
    fn execute_open(&self) -> CommandResult<()> {
        let selected_bat_metadata_type =
            BatMetadataType::prompt_metadata_type_selection().change_context(CommandError)?;
        let (path, start_line_index) = match selected_bat_metadata_type {
            BatMetadataType::Struct => {
                let StructMetadata {
                    path,
                    start_line_index,
                    ..
                } = StructMetadata::prompt_selection().change_context(CommandError)?;
                (path, start_line_index)
            }
            BatMetadataType::Function => {
                let FunctionMetadata {
                    path,
                    start_line_index,
                    ..
                } = FunctionMetadata::prompt_selection().change_context(CommandError)?;
                (path, start_line_index)
            }
            BatMetadataType::Trait => {
                let TraitMetadata {
                    path,
                    start_line_index,
                    ..
                } = TraitMetadata::prompt_selection().change_context(CommandError)?;
                (path, start_line_index)
            }
        };
        CodeEditor::open_file_in_editor(&path, Some(start_line_index))
            .change_context(CommandError)?;
        Ok(())
    }

    fn execute_package_json(&self) -> CommandResult<()> {
        let prompt_text = format!("Select the log level:");
        let log_level_vec = vec![
            Level::Warn,
            Level::Info,
            Level::Debug,
            Level::Trace,
            Level::Error,
        ];
        let selection = BatDialoguer::select(
            prompt_text,
            log_level_vec
                .clone()
                .into_iter()
                .enumerate()
                .map(|(idx, level)| ToolsCommands::colored_from_index(&level.to_string(), idx))
                .collect::<Vec<_>>(),
            None,
        )?;
        let level_selected = log_level_vec[selection].clone();
        PackageJsonTemplate::create_package_json(Some(level_selected))
            .change_context(CommandError)?;
        BatFile::PackageJson {
            to_create_project: false,
        }
        .open_in_editor(false, None)
        .change_context(CommandError)
    }
}
