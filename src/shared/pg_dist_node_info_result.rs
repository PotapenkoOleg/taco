#[derive(Debug, Clone)]
pub struct  PgDistNodeInfoResult {
    pub nodeid: Option<i32>,
    pub groupid: Option<i32>,
    pub nodename: Option<String>,
    pub nodeport: Option<i32>,
    pub noderack: Option<String>,
    pub hasmetadata: Option<bool>,
    pub isactive: Option<bool>,
    pub noderole: Option<String>,
    pub nodecluster: Option<String>,
    pub metadatasynced: Option<bool>,
    pub shouldhaveshards: Option<bool>,
}
