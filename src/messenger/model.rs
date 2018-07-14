#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    pub vanity: String,
    #[serde(rename = "thumbSrc")]
    pub thumb_src: String,
    #[serde(rename = "profileUrl")]
    pub profile_url: String,
    pub gender: i32,
    #[serde(rename = "type")]
    pub utype: String,
    #[serde(rename = "isFriend")]
    pub is_friend: bool,
    #[serde(rename = "isBirthday")]
    pub is_birthday: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    #[serde(rename = "type")]
    pub mtype: String,
    pub attachments: Vec<Attachment>,
    pub body: String,
    #[serde(rename = "isGroup")]
    pub is_group: bool,
    #[serde(rename = "messageID")]
    pub message_id: String,
    #[serde(rename = "senderID")]
    pub sender_id: String,
    #[serde(rename = "threadID")]
    pub thread_id: String,
    pub timestamp: String,
    #[serde(rename = "isUnread")]
    pub is_unread: bool,
    #[serde(rename = "isSponsored")]
    pub is_sponsored: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attachment {
    pub name: String,
    #[serde(rename = "type")]
    pub atype: String,
    pub filename: String,
    #[serde(rename = "ID")]
    pub id: String,
    pub url: String,
    #[serde(rename = "isMalicious")]
    pub is_malicious: bool,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(rename = "fileSize")]
    pub file_size: i32,
}
