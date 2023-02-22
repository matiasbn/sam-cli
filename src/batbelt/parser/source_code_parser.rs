use std::fs;

use colored::Colorize;

use crate::batbelt::metadata::BatMetadataType;
use crate::batbelt::miro::frame::MiroFrame;
use crate::batbelt::miro::image::MiroImage;
use crate::batbelt::miro::item::MiroItem;
use crate::batbelt::miro::MiroItemType;
use crate::batbelt::parser::ParserError;
use crate::batbelt::path::BatFile;
use crate::batbelt::silicon;
use crate::batbelt::sonar::BatSonar;
use crate::batbelt::{self, path::BatFolder};
use crate::config::BatConfig;
use error_stack::{Result, ResultExt};

#[derive(Debug, Clone)]
pub struct SourceCodeScreenshotOptions {
    pub include_path: bool,
    pub offset_to_start_line: bool,
    pub filter_comments: bool,
    pub font_size: Option<usize>,
    pub filters: Option<Vec<String>>,
    pub show_line_number: bool,
}

impl SourceCodeScreenshotOptions {
    pub fn get_default_metadata_options(metadata_section: BatMetadataType) -> Self {
        match metadata_section {
            BatMetadataType::Structs => Self {
                include_path: true,
                offset_to_start_line: true,
                filter_comments: false,
                font_size: Some(20),
                filters: None,
                show_line_number: true,
            },
            BatMetadataType::Functions => Self {
                include_path: true,
                offset_to_start_line: true,
                filter_comments: false,
                font_size: Some(20),
                filters: None,
                show_line_number: true,
            },
            BatMetadataType::Entrypoints => Self {
                include_path: true,
                offset_to_start_line: true,
                filter_comments: false,
                font_size: Some(20),
                filters: None,
                show_line_number: true,
            },
            // BatMetadataType::Miro => Self {
            //     include_path: true,
            //     offset_to_start_line: true,
            //     filter_comments: false,
            //     font_size: Some(20),
            //     filters: None,
            //     show_line_number: true,
            // },
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceCodeParser {
    pub name: String,
    pub path: String,
    pub start_line_index: usize,
    pub end_line_index: usize,
}

impl SourceCodeParser {
    pub fn new(name: String, path: String, start_line_index: usize, end_line_index: usize) -> Self {
        SourceCodeParser {
            name,
            path,
            start_line_index,
            end_line_index,
        }
    }

    pub fn get_source_code_content(&self) -> String {
        let content = fs::read_to_string(&self.path).unwrap();
        let content_lines = content.lines().collect::<Vec<_>>()
            [self.start_line_index - 1..=self.end_line_index - 1]
            .to_vec()
            .join("\n");
        content_lines
    }

    pub fn get_entrypoints_sourcecode() -> Result<Vec<Self>, ParserError> {
        let lib_file_path =
            batbelt::path::get_file_path(BatFile::ProgramLib, false).change_context(ParserError)?;
        let entrypoints = BatSonar::get_entrypoints_results().change_context(ParserError)?;
        let sourcecodes = entrypoints
            .results
            .into_iter()
            .map(|res| {
                SourceCodeParser::new(
                    res.name,
                    lib_file_path.clone(),
                    res.start_line_index,
                    res.end_line_index,
                )
            })
            .collect::<Vec<_>>();
        Ok(sourcecodes)
    }

    // pub fn new_from_metadata_data(name: &str, section: &str, subsection: &str) -> Self {
    //     let metadata_path = batbelt::path::get_file_path(FilePathType::Metadata, true);
    //     let metadata_markdown = MarkdownFile::new(&metadata_path);
    //     let section = metadata_markdown.get_section(section).unwrap();
    //     let subsection = section.borrow().get_subsection_by_title(subsection);
    //     let source_code_metadata = subsection.borrow().get_subsection_by_title(name);
    //     let path = Self::parse_metadata_info_section(
    //         &source_code_metadata.borrow().content,
    //         MetadataContent::Path.get_prefix(),
    //     );
    //     let start_line_index: usize = Self::parse_metadata_info_section(
    //         &source_code_metadata.borrow().content,
    //         MetadataContent::StartLineIndex.get_prefix(),
    //     )
    //     .parse()
    //     .unwrap();
    //     let end_line_index: usize = Self::parse_metadata_info_section(
    //         &source_code_metadata.borrow().content,
    //         MetadataContent::EndLineIndex.get_prefix(),
    //     )
    //     .parse()
    //     .unwrap();
    //     SourceCodeMetadata::new(name.to_string(), path, start_line_index, end_line_index)
    // }

    pub fn create_screenshot(
        &self,
        options: SourceCodeScreenshotOptions,
    ) -> Result<String, ParserError> {
        let dest_path = batbelt::path::get_folder_path(BatFolder::AuditorFigures, true)
            .change_context(ParserError)?;
        let mut offset = if options.offset_to_start_line {
            self.start_line_index
        } else {
            0
        };
        let content = fs::read_to_string(&self.path).unwrap();
        let content_lines = content.lines().collect::<Vec<_>>()
            [self.start_line_index - 1..=self.end_line_index - 1]
            .to_vec();
        let content_vec = content_lines
            .iter()
            .filter_map(|line| {
                if options.filter_comments {
                    let starts_with_comment = line.trim().split(" ").next().unwrap().contains("//");
                    if starts_with_comment {
                        return None;
                    }
                    // at this point it does not start with comment
                    let line_contains_comment = line.contains("//");
                    if line_contains_comment {
                        return Some(line.split("//").next().unwrap().to_string());
                    }
                }
                return Some(line.to_string());
            })
            .filter(|line| {
                if let Some(filters) = options.filters.clone() {
                    !filters.into_iter().any(|filter| line.contains(&filter))
                } else {
                    true
                }
            })
            .collect::<Vec<_>>()
            .to_vec();
        let mut content = content_vec.join("\n");
        if options.include_path {
            log::debug!("self:\n{:#?}", self);
            let bat_config = BatConfig::get_config().change_context(ParserError)?;
            log::debug!("program_name: {}", bat_config.program_name);
            let splitter = format!("{}/src/", bat_config.program_name);
            log::debug!("splitter: {}", splitter);
            let path = self.path.split(&splitter).last().unwrap();
            log::debug!("splitted_path_lasth: {}", path);
            let path_to_include = format!("{}{}", splitter, path)
                .trim_start_matches("/")
                .to_string();
            log::debug!("path_to_include: {}", path_to_include);
            content = format!("// {}\n\n{}", path_to_include, content);
            offset = if options.offset_to_start_line {
                offset - 2
            } else {
                0
            };
        }

        let png_screenshot_path = silicon::create_figure(
            &content,
            &dest_path,
            &self.name,
            offset,
            options.font_size,
            options.show_line_number,
        );
        Ok(png_screenshot_path)
    }

    pub fn prompt_screenshot_options() -> SourceCodeScreenshotOptions {
        let include_path = batbelt::cli_inputs::select_yes_or_no(&format!(
            "Do you want to {}",
            "include the path?".yellow()
        ))
        .unwrap();
        let filter_comments = batbelt::cli_inputs::select_yes_or_no(&format!(
            "Do you want to {}",
            "filter the comments?".yellow()
        ))
        .unwrap();
        let show_line_number = batbelt::cli_inputs::select_yes_or_no(&format!(
            "Do you want to {}",
            "include the line numbers?".yellow()
        ))
        .unwrap();
        let offset_to_start_line = if show_line_number {
            batbelt::cli_inputs::select_yes_or_no(&format!(
                "Do you want to {}",
                "offset to the starting line?".yellow()
            ))
            .unwrap()
        } else {
            false
        };
        let include_filters = batbelt::cli_inputs::select_yes_or_no(&format!(
            "Do you want to {}",
            "add customized filters?".red()
        ))
        .unwrap();
        // utils::cli_inputs::select_yes_or_no("Do you want to include filters?").unwrap();
        let filters = if include_filters {
            let filters_to_include = batbelt::cli_inputs::input(
                "Please enter the filters, comma separated: #[account,CHECK ",
            )
            .unwrap();
            if !filters_to_include.is_empty() {
                let filters: Vec<String> = filters_to_include
                    .split(",")
                    .map(|filter| filter.trim().to_string())
                    .collect();
                Some(filters)
            } else {
                None
            }
        } else {
            None
        };
        let screenshot_options = SourceCodeScreenshotOptions {
            include_path,
            offset_to_start_line,
            filter_comments,
            show_line_number,
            filters,
            font_size: Some(20),
        };
        screenshot_options
    }

    pub async fn deploy_screenshot_to_miro_frame(
        &self,
        miro_frame: MiroFrame,
        x_position: i64,
        y_position: i64,
        options: SourceCodeScreenshotOptions,
    ) -> Result<MiroImage, ParserError> {
        let png_path = self.create_screenshot(options.clone())?;
        let miro_frame_id = miro_frame.item_id.clone();
        println!(
            "\nCreating {}{} in {} frame",
            self.name.green(),
            ".png".green(),
            miro_frame.title.green()
        );
        let mut screenshot_image =
            MiroImage::new_from_file_path(&png_path, &miro_frame.item_id.clone());
        screenshot_image
            .deploy()
            .await
            .change_context(ParserError)?;
        let miro_item = MiroItem::new(
            &screenshot_image.item_id,
            &miro_frame_id,
            x_position,
            y_position,
            MiroItemType::Image,
        );
        println!(
            "Updating the position of {}{}\n",
            self.name.green(),
            ".png".green()
        );
        fs::remove_file(png_path).unwrap();
        miro_item.update_item_parent_and_position().await;
        Ok(screenshot_image)
    }

    fn parse_metadata_info_section(metadata_info_content: &str, section: &str) -> String {
        let path = metadata_info_content
            .lines()
            .find(|line| line.contains(section))
            .unwrap()
            .replace(section, "")
            .trim()
            .to_string();
        path
    }
}

#[test]
fn test_filter_comments() {
    let test_text = "this part should not be filtered //this part should be filtered
    // this line should be completely filtered

    previous line is empty
    ";
    let test_text_lines = test_text.lines();
    let filtered_lines = test_text_lines
        .filter_map(|line| {
            if !line.is_empty() {
                let starts_with_comment = line.trim().split(" ").next().unwrap().contains("//");
                if starts_with_comment {
                    return None;
                }
                // at this point it does not start with comment
                let line_contains_comment = line.contains("//");
                if line_contains_comment {
                    return Some(line.split("//").next().unwrap());
                }
            }
            return Some(line);
        })
        .collect::<Vec<_>>()
        .join("\n");

    println!("filtered \n{}", filtered_lines)
}

#[test]
fn test_include_path() {
    let test_path = "../star-atlas-programs/sol-programs/programs/cargo/src/instructions/update_token_account_for_invalid_type.rs";
    let test_program_name = "cargo/src/";
    let path = test_path.split(test_program_name).last().unwrap();
    println!("path {}", path);
    let path = format!("{}{}", test_program_name, path);
    println!("path {}", path)
}