use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 上传文件API的响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    /// 状态码，0表示成功，-1表示密钥无效
    pub code: i32,
    /// 错误消息
    pub msg: Option<String>,
    /// 上传结果数据
    pub data: Option<UploadData>,
}

/// 上传文件结果数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadData {
    /// 上传失败的文件名列表
    #[serde(rename = "errFiles")]
    pub err_files: Vec<String>,
    /// 上传成功的文件信息映射
    #[serde(rename = "succMap")]
    pub succ_map: HashMap<String, String>,
}
