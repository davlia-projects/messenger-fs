#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    name: String,
    #[serde(rename = "firstName")]
    first_name: String,
    vanity: String,
    #[serde(rename = "thumbSrc")]
    thumb_src: String,
    #[serde(rename = "profileUrl")]
    profile_url: String,
    gender: i32,
    #[serde(rename = "type")]
    utype: String,
    #[serde(rename = "isFriend")]
    is_friend: bool,
    #[serde(rename = "isBirthday")]
    is_birthday: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    #[serde(rename = "type")]
    mtype: String,
    attachments: Vec<Attachment>,
    body: String,
    #[serde(rename = "isGroup")]
    is_group: bool,
    #[serde(rename = "messageID")]
    message_id: String,
    #[serde(rename = "senderID")]
    sender_id: String,
    #[serde(rename = "threadID")]
    thread_id: String,
    timestamp: String,
    #[serde(rename = "isUnread")]
    is_unread: bool,
    #[serde(rename = "isSponsored")]
    is_sponsored: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attachment {
    name: String,
    #[serde(rename = "type")]
    atype: String,
    filename: String,
    #[serde(rename = "ID")]
    id: String,
    url: String,
    #[serde(rename = "isMalicious")]
    is_malicious: bool,
    #[serde(rename = "contentType")]
    content_type: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
    #[serde(rename = "fileSize")]
    file_size: i32,
}
