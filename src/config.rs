use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cluster {
    #[serde(
        rename = "certificate-authority-data",
        skip_serializing_if = "Option::is_none"
    )]
    pub certificate_authority_data: Option<String>,
    #[serde(
        rename = "certificate-authority",
        skip_serializing_if = "Option::is_none"
    )]
    pub certificate_authority: Option<String>,
    pub server: String,
    #[serde(
        rename = "insecure-skip-tls-verify",
        skip_serializing_if = "Option::is_none"
    )]
    pub insecure_skip_tls_verify: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_yml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamedCluster {
    pub name: String,
    pub cluster: Cluster,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Context {
    pub cluster: String,
    pub user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_yml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamedContext {
    pub name: String,
    pub context: Context,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(
        rename = "client-certificate-data",
        skip_serializing_if = "Option::is_none"
    )]
    pub client_certificate_data: Option<String>,
    #[serde(rename = "client-key-data", skip_serializing_if = "Option::is_none")]
    pub client_key_data: Option<String>,
    #[serde(rename = "client-certificate", skip_serializing_if = "Option::is_none")]
    pub client_certificate: Option<String>,
    #[serde(rename = "client-key", skip_serializing_if = "Option::is_none")]
    pub client_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_yml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamedUser {
    pub name: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KubeConfig {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clusters: Option<Vec<NamedCluster>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<NamedContext>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<NamedUser>>,
    #[serde(rename = "current-context", skip_serializing_if = "String::is_empty")]
    pub current_context: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub preferences: HashMap<String, serde_yml::Value>,
}
