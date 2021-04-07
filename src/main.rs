use std::env;
use std::result::Result;
use std::sync::Arc;

mod rest;
mod lib;
mod k8s;

use crate::lib::Cluster;

fn main() -> Result<(), String> {
    let url = error_handler!(env::var("APISERVER"), "The environment variable APISERVER is not set");
    let certificate = error_handler!(env::var("APICERT"), "The environment variable APICERT is not set");
    let token = error_handler!(env::var("TOKEN"), "The environment variable TOKEN is not set");
    
    let cluster_instance = Cluster {
        address: url.to_string(),
        token: token.to_string(),
        certificate: certificate.to_string(),
    };
    
    let arc_cluster = Arc::new(cluster_instance);
    
    let api_resources = match k8s::fetch_available_resources(arc_cluster.clone()) {
        Ok(ai) => ai,
        Err(e) => return Err(format!("Unable to get the API resources: {}", e)),
    };
    
    return k8s::watch_resource_patcher_objects(arc_cluster, api_resources);
}
