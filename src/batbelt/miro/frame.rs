use super::*;
use crate::batbelt::miro::helpers::get_id_from_response;

pub struct MiroFrame {
    pub title: String,
    pub item_id: String,
    pub frame_url: Option<String>,
    pub height: u64,
    pub width: u64,
    pub x_position: i64,
    pub y_position: i64,
}

impl MiroFrame {
    pub fn new(title: &str, height: u64, width: u64, x_position: i64, y_position: i64) -> Self {
        MiroFrame {
            title: title.to_string(),
            item_id: "".to_string(),
            frame_url: None,
            height,
            width,
            x_position,
            y_position,
        }
    }

    pub async fn new_from_item_id(item_id: &str) -> Self {
        let miro_config = MiroConfig::new();
        let response = MiroItem::get_specific_item_on_board(item_id).await.unwrap();
        let response_string = response.text().await.unwrap();
        let response: Value = serde_json::from_str(&&response_string.as_str()).unwrap();
        let item_id = response["id"].to_string().replace("\"", "");
        let frame_url = miro_config.get_frame_url(&item_id);
        let title = response["data"]["title"].to_string().replace("\"", "");
        let height = response["geometry"]["height"].as_f64().unwrap() as u64;
        let width = response["geometry"]["width"].as_f64().unwrap() as u64;
        let x_position = response["position"]["x"].as_f64().unwrap() as i64;
        let y_position = response["position"]["y"].as_f64().unwrap() as i64;
        MiroFrame {
            title: title,
            item_id,
            frame_url: Some(frame_url),
            height,
            width,
            x_position,
            y_position,
        }
    }

    pub async fn deploy(&mut self) -> Result<(), String> {
        let id = api::create_frame(
            &self.title,
            self.x_position,
            self.y_position,
            self.width,
            self.height,
        )
        .await?;
        self.item_id = id.clone();
        self.frame_url = Some(MiroConfig::new().get_frame_url(&id));
        Ok(())
    }

    pub async fn get_frames_from_miro() -> Vec<MiroFrame> {
        let response = MiroItem::get_items_on_board(Some(MiroItemType::Frame))
            .await
            .unwrap();
        let response_string = response.text().await.unwrap();
        let response: Value = serde_json::from_str(&&response_string.as_str()).unwrap();
        let data = response["data"].as_array().unwrap();
        // println!("data {:#?}", data);
        let miro_config = MiroConfig::new();
        let frames = data
            .clone()
            .into_iter()
            .map(|data_response| {
                let item_id = data_response["id"].to_string().replace("\"", "");
                let frame_url = miro_config.get_frame_url(&item_id);
                let title = data_response["data"]["title"].to_string().replace("\"", "");
                let height = data_response["geometry"]["height"].as_f64().unwrap() as u64;
                let width = data_response["geometry"]["width"].as_f64().unwrap() as u64;
                let x_position = data_response["position"]["x"].as_f64().unwrap() as i64;
                let y_position = data_response["position"]["y"].as_f64().unwrap() as i64;
                let mut miro_frame = MiroFrame::new(&title, height, width, x_position, y_position);
                miro_frame.item_id = item_id;
                miro_frame.frame_url = Some(frame_url);
                miro_frame
            })
            .collect();
        frames
    }

    pub async fn update_position(
        &mut self,
        x_position: i64,
        y_position: i64,
    ) -> Result<(), String> {
        api::update_frame_position(&self.item_id, x_position, y_position);
        self.x_position = x_position;
        self.y_position = y_position;
        Ok(())
    }
}

mod api {
    use super::*;

    // returns the frame url
    pub async fn create_frame(
        frame_title: &str,
        x_position: i64,
        y_position: i64,
        width: u64,
        height: u64,
    ) -> Result<String, String> {
        let MiroConfig {
            access_token,
            board_id,
            ..
        } = MiroConfig::new();
        let client = reqwest::Client::new();

        let response = client
            .post(format!("https://api.miro.com/v2/boards/{board_id}/frames"))
            .body(
                json!({
                     "data": {
                          "format": "custom",
                          "title": frame_title,
                          "type": "freeform"
                     },
                     "position": {
                          "origin": "center",
                          "x": x_position,
                          "y": y_position
                     },
                     "geometry": {
                        "width": width,
                        "height": height
                   }
                })
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .send()
            .await;
        let id = get_id_from_response(response).await?;
        Ok(id)
    }

    // returns the frame url
    pub async fn create_frame_for_entrypoint(entrypoint_name: &str) -> Result<String, String> {
        let MiroConfig {
            access_token,
            board_id,
            ..
        } = MiroConfig::new();
        let client = reqwest::Client::new();

        let board_response = client
            .post(format!("https://api.miro.com/v2/boards/{board_id}/frames"))
            .body(
                json!({
                     "data": {
                          "format": "custom",
                          "title": entrypoint_name,
                          "type": "freeform"
                     },
                     "position": {
                          "origin": "center",
                          "x": 0,
                          "y": 0
                     },
                     "geometry": {
                        "width": MIRO_FRAME_WIDTH,
                        "height": MIRO_FRAME_HEIGHT
                   }
                })
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .send()
            .await;
        let id = get_id_from_response(board_response).await?;
        Ok(id)
    }

    pub async fn get_frame_position(frame_id: &str) -> (u64, u64) {
        let MiroConfig {
            access_token,
            board_id,
            ..
        } = MiroConfig::new();
        let client = reqwest::Client::new();
        let board_response = client
            .get(format!(
                "https://api.miro.com/v2/boards/{board_id}/frames/{frame_id}"
            ))
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let response: Value = serde_json::from_str(board_response.as_str()).unwrap();
        let x_position = response["position"]["x"].clone();
        let y_position = response["position"]["y"].clone();
        (
            x_position.as_f64().unwrap() as u64,
            y_position.as_f64().unwrap() as u64,
        )
    }

    pub async fn update_frame_position(
        frame_id: &str,
        x_position: i64,
        y_position: i64,
    ) -> Result<(), String> {
        let MiroConfig {
            access_token,
            board_id,
            ..
        } = MiroConfig::new();
        let client = reqwest::Client::new();
        let _response = client
            .patch(format!(
                "https://api.miro.com/v2/boards/{board_id}/frames/{frame_id}",
            ))
            .body(
                json!({
                    "position": {
                        "x": x_position,
                        "y": y_position,
                        "origin": "center",
                    },
                })
                .to_string(),
            )
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        Ok(())
        // println!("update frame position response: {response}")
    }

    pub async fn get_items_within_frame(frame_id: &str) -> (u64, u64) {
        let MiroConfig {
            access_token,
            board_id,
            ..
        } = MiroConfig::new();
        let client = reqwest::Client::new();
        let board_response = client
            .get(format!(
                "https://api.miro.com/v2/boards/{board_id}/frames/{frame_id}"
            ))
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let response: Value = serde_json::from_str(board_response.as_str()).unwrap();
        let x_position = response["position"]["x"].clone();
        let y_position = response["position"]["y"].clone();
        (
            x_position.as_f64().unwrap() as u64,
            y_position.as_f64().unwrap() as u64,
        )
    }

    // pub async fn update_frame_position(
    //     entrypoint_name: String,
    //     co_finished_files: i32,
    // ) -> Result<(), String> {
    //     let MiroConfig {
    //         access_token,
    //         board_id,
    //         ..
    //     } = MiroConfig::new();
    //     let frame_id = super::helpers::get_frame_id_from_co_file(entrypoint_name.as_str())?;
    //     let x_modifier = co_finished_files % MIRO_BOARD_COLUMNS;
    //     let y_modifier = co_finished_files / MIRO_BOARD_COLUMNS;
    //     let x_position = MIRO_INITIAL_X + (MIRO_FRAME_WIDTH + 100) * x_modifier;
    //     let y_position = MIRO_INITIAL_Y + (MIRO_FRAME_HEIGHT + 100) * y_modifier;
    //     let client = reqwest::Client::new();
    //     let _response = client
    //         .patch(format!(
    //             "https://api.miro.com/v2/boards/{board_id}/frames/{frame_id}",
    //         ))
    //         .body(
    //             json!({
    //                 "position": {
    //                     "x": x_position,
    //                     "y": y_position,
    //                     "origin": "center",
    //                 },
    //             })
    //                 .to_string(),
    //         )
    //         .header(CONTENT_TYPE, "application/json")
    //         .header(AUTHORIZATION, format!("Bearer {access_token}"))
    //         .send()
    //         .await
    //         .unwrap()
    //         .text()
    //         .await
    //         .unwrap();
    //     Ok(())
    //     // println!("update frame position response: {response}")
    // }
}