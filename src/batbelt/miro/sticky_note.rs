use crate::batbelt::miro::{MiroColor, MiroConfig, MiroItemType};

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};

#[derive(Clone)]
pub struct MiroStickyNote {
    pub content: String,
    pub color: MiroColor,
    pub parent_id: String,
    pub item_type: MiroItemType,
    pub item_id: String,
    pub x_position: i64,
    pub y_position: i64,
    pub width: u64,
}

impl MiroStickyNote {
    pub fn new(
        content: &str,
        color: MiroColor,
        parent_id: &str,
        x_position: i64,
        y_position: i64,
        width: u64,
    ) -> Self {
        MiroStickyNote {
            content: content.to_string(),
            color,
            parent_id: parent_id.to_string(),
            item_type: MiroItemType::StickyNote,
            item_id: "".to_string(),
            x_position,
            y_position,
            width,
        }
    }

    pub async fn deploy(&mut self) {
        let id = api::create_sticky_note(
            &self.content,
            self.color.clone().to_str(),
            &self.parent_id,
            self.x_position,
            self.y_position,
            self.width,
        )
        .await;
        self.item_id = id;
    }
}

mod api {
    use super::*;
    pub async fn create_sticky_note(
        content: &str,
        color: &str,
        parent_id: &str,
        x_position: i64,
        y_position: i64,
        width: u64,
    ) -> String {
        let MiroConfig {
            access_token,
            board_id,
            ..
        } = MiroConfig::new();
        // let x_position = x + x_move;
        let client = reqwest::Client::new();
        let response = client
            .post(format!(
                "https://api.miro.com/v2/boards/{board_id}/sticky_notes",
            ))
            .body(
                json!({
                    "data": {
                        "content": content,
                        "shape": "rectangle"
                    },
                    "style": {
                        "fillColor": color,
                    },
                    "position": {
                        "origin": "center",
                        "x": x_position,
                        "y": y_position
                    },
                    "geometry": {
                        "width": width
                    },
                    "parent": {
                        "id": parent_id
                    }
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
        // println!("sticky not response {response}");
        let response: Value = serde_json::from_str(&response.as_str()).unwrap();
        let id = response["id"].to_string().replace("\"", "");
        id
    }

    //     pub async fn create_signer_sticky_note(
    //         signer_note_text: String,
    //         signer_counter: usize,
    //         miro_frame_id: String,
    //         signer_type: SignerType,
    //     ) -> String {
    //         let MiroConfig {
    //             access_token,
    //             board_id,
    //             ..
    //         } = MiroConfig::new();
    //         // let x_position = x + x_move;
    //         let client = reqwest::Client::new();
    //         let y_position = 150 + signer_counter * 270;
    //         let fill_color = match signer_type {
    //             SignerType::Validated => "red",
    //             SignerType::NotValidated => "dark_blue",
    //             SignerType::NotSigner => "gray",
    //         };
    //         let response = client
    //             .post(format!(
    //                 "https://api.miro.com/v2/boards/{board_id}/sticky_notes",
    //             ))
    //             .body(
    //                 json!({
    //                 "data": {
    //                     "content": signer_note_text,
    //                     "shape": "rectangle"
    //                 },
    //                 "style": {
    //                     "fillColor": fill_color
    //                 },
    //                 "position": {
    //                     "origin": "center",
    //                     "x": 550,
    //                     "y": y_position
    //                 },
    //                 "geometry": {
    //                     "width": 374.5
    //                 },
    //                 "parent": {
    //                     "id": miro_frame_id
    //                 }
    //             })
    //                     .to_string(),
    //             )
    //             .header(CONTENT_TYPE, "application/json")
    //             .header(AUTHORIZATION, format!("Bearer {access_token}"))
    //             .send()
    //             .await
    //             .unwrap()
    //             .text()
    //             .await
    //             .unwrap();
    //         // println!("sticky not response {response}");
    //         let response: Value = serde_json::from_str(&response.as_str()).unwrap();
    //         let id = response["id"].to_string().replace("\"", "");
    //         id
    //     }
    //
}
