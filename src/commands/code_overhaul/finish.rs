use colored::Colorize;

use crate::batbelt::constants::{
    MIRO_BOARD_COLUMNS, MIRO_FRAME_HEIGHT, MIRO_FRAME_WIDTH, MIRO_INITIAL_X, MIRO_INITIAL_Y,
};
use crate::batbelt::miro::MiroConfig;

use crate::batbelt;
use crate::batbelt::git::{check_correct_branch, create_git_commit, GitCommit};

use crate::batbelt::path::{FilePathType, FolderPathType};

use crate::batbelt::miro::frame::MiroFrame;

use std::process::Command;
use std::string::String;

pub async fn finish_co_file() -> Result<(), String> {
    check_correct_branch()?;
    // get to-review files
    let started_endpoints = batbelt::helpers::get::get_started_entrypoints()?;
    let prompt_text = "Select the code-overhaul to finish:";
    let selection = batbelt::cli_inputs::select(prompt_text, started_endpoints.clone(), None)?;

    let finished_endpoint = &started_endpoints[selection].clone();
    let finished_co_folder_path =
        batbelt::path::get_folder_path(FolderPathType::CodeOverhaulFinished, true);
    let started_co_file_path = batbelt::path::get_file_path(
        FilePathType::CodeOverhaulStarted {
            file_name: finished_endpoint.clone(),
        },
        true,
    );
    batbelt::helpers::check::code_overhaul_file_completed(
        started_co_file_path.clone(),
        finished_endpoint.clone(),
    );

    Command::new("mv")
        .args([started_co_file_path, finished_co_folder_path])
        .output()
        .unwrap();

    // if MiroConfig::new().miro_enabled() {
    //     let (_, _, finished_co) = batbelt::helpers::count::co_counter()?;
    //     let frame_id =
    //         batbelt::miro::helpers::get_frame_id_from_co_file(finished_endpoint.as_str())?;
    //     let mut frame = MiroFrame::new_from_item_id(&frame_id).await;
    //     let x_modifier = finished_co as i64 % MIRO_BOARD_COLUMNS;
    //     let y_modifier = finished_co as i64 / MIRO_BOARD_COLUMNS;
    //     let x_position = MIRO_INITIAL_X + (MIRO_FRAME_WIDTH as i64 + 100) * x_modifier;
    //     let y_position = MIRO_INITIAL_Y + (MIRO_FRAME_HEIGHT as i64 + 100) * y_modifier;
    //     frame.update_position(x_position, y_position).await?;
    //     let started_co_folder_path =
    //         batbelt::path::get_folder_path(FolderPathType::CodeOverhaulStarted, true);
    //     let started_co_subfolder_path = format!("{}/{}", started_co_folder_path, finished_endpoint);
    //     let started_co_subfolder_path = format!("{}/{}", started_co_folder_path, finished_endpoint);
    //
    //     // remove co subfolder
    //     Command::new("rm")
    //         .args(["-rf", &started_co_subfolder_path])
    //         .output()
    //         .unwrap();
    //
    //     create_git_commit(
    //         GitCommit::FinishCOMiro,
    //         Some(vec![finished_endpoint.to_string()]),
    //     )?;
    // } else {
    // }
    create_git_commit(
        GitCommit::FinishCO,
        Some(vec![finished_endpoint.to_string()]),
    )?;

    println!("{} moved to finished", finished_endpoint.green());
    Ok(())
}
