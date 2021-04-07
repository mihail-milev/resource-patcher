use serde::Deserialize;
use std::vec::Vec;

#[macro_export]
macro_rules! error_handler {
    ($a:expr, $b:expr) => {
        match $a {
            Ok(v) => v,
            Err(e) => {
                return Err(format!("{}: {}", $b, e))
            },
        }
    }
}

pub struct Cluster {
    pub address: String,
    pub token: String,
    pub certificate: String,
}

/*
//
// MAIN FUNCTIONALITY
//
*/

#[derive(Deserialize)]
pub struct ResourcePatchList {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub items: Vec<ResourcePatch>,
}

#[derive(Deserialize)]
pub struct ResourcePatch {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: Spec,
}

#[derive(Deserialize)]
pub struct Metadata {
    pub name: String,
    #[serde(default)]
    pub namespace: String,
    #[serde(default)]
    #[serde(rename = "resourceVersion")]
    pub resource_version: String,
}

#[derive(Deserialize)]
pub struct Spec {
    #[serde(rename = "targetObj")]
    pub target_obj: TargetObject,
    pub patch: String,
}

#[derive(Deserialize)]
pub struct TargetObject {
    pub name: String,
    #[serde(default)]
    pub namespace: String,
    pub kind: String,
    #[serde(rename = "apiVersion")]
    pub api_version: String,
}

#[derive(Deserialize)]
pub struct GenericItem {
    pub kind: String,
    pub metadata: Metadata,
}


/*
//
// FETCH API RESOURCES
//
*/

#[derive(Deserialize)]
pub struct ApiGroupList {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub groups: Vec<ApiGroupListItem>,
}

#[derive(Deserialize)]
pub struct ApiGroupListItem {
    pub name: String,
    #[serde(rename = "preferredVersion")]
    pub preferred_version: PreferredVersion,
}

#[derive(Deserialize)]
pub struct PreferredVersion {
    #[serde(rename = "groupVersion")]
    pub group_version: String,
    pub version: String,
}

#[derive(Deserialize)]
pub struct ApiResourceList {
    pub kind: String,
    pub resources: Vec<ApiResourceListItem>,
}

#[derive(Deserialize)]
pub struct ApiResourceListItem {
    pub name: String,
    pub kind: String,
}
