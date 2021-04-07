use std::thread;
use std::result::Result;
use std::boxed::Box;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Arc;
use serde_yaml::Value;
use tokio::runtime::Runtime;
use futures::future;

use crate::lib::{Cluster, ResourcePatchList, ResourcePatch, ApiGroupList, ApiGroupListItem, PreferredVersion, ApiResourceList, GenericItem};
use crate::error_handler;
use crate::rest::{perform_get_request, perform_patch_request};

const URL_LIST_RESOURCE_PATCH_OBJECTS : &str = "apis/mmilev.io/v1alpha1/resourcepatches";
const URL_LIST_ALL_AVAILABLE_RESOURCES : &str = "apis";
const RESOURCE_PATCH_OBJECT_REFRESH_INTERVAL_MS : u64 = 5000;

/*
//
// MAIN FUNCTIONALITY
//
*/

pub fn watch_resource_patcher_objects(cluster: Arc<Cluster>, api_resources: Arc<HashMap<String, String>>) -> Result<(), String> {
    let rt = error_handler!(Runtime::new(), "Unable to start multithreading runtime");
    let full_get_url = format!("{}/{}", cluster.address, URL_LIST_RESOURCE_PATCH_OBJECTS);
    loop {
        let start_time = Instant::now();
        match perform_get_request(&full_get_url, &(cluster.token), &(cluster.certificate)) {
            Ok(t) => {
                let _nointerest = work_on_resource_patch_list(t, &rt, &cluster, &api_resources);
            },
            Err(e) => eprintln!("Unable to get ResourcePatchList: {}", e),
        };
        let time_worked = start_time.elapsed();
        let sleep_int = Duration::from_millis(RESOURCE_PATCH_OBJECT_REFRESH_INTERVAL_MS) - time_worked;
        thread::sleep(sleep_int);
    }
}

fn work_on_resource_patch_list(list: String, runtime: &Runtime, cluster: &Arc<Cluster>, api_resources: &Arc<HashMap<String, String>>) -> Result<(), String> {
    let rpl: ResourcePatchList = error_handler!(serde_json::from_str(&list), "Unable to parse result into ResourcePatchList".to_string());
    println!("ResourcePatch items found: {}", rpl.items.len());
    for subitem in rpl.items {
        let ac = cluster.clone(); // cloning an Arc -> points to same heap space, only counter is increased
        let ar = api_resources.clone(); // cloning an Arc -> points to same heap space, only counter is increased
        let _subproc = runtime.spawn(async {
            match work_on_subitem(subitem, ac, ar) {
                Ok(_) => {},
                Err(e) => eprintln!("Problem when parsing item: {}", e),
            };
        });
    }
    Ok(())
}

fn work_on_subitem(subitem: ResourcePatch, cluster: Arc<Cluster>, api_resources: Arc<HashMap<String, String>>) -> Result<(), String> {
    if subitem.kind != "ResourcePatch" {
        return Err("Item is not a ResourcePatch".to_string());
    }
    if subitem.spec.target_obj.name == "" {
        return Err("Target object's name may not be empty".to_string());
    }
    if subitem.spec.target_obj.api_version == "" { // TODO: check proper api_version format
        return Err("Target object's apiVersion may not be empty".to_string());
    }
    if subitem.spec.target_obj.kind == "" {
        return Err("Target object's kind may not be empty".to_string());
    }
    let search_kind = {
        match api_resources.get(&subitem.spec.target_obj.kind) {
            Some(v) => v.clone(),
            None => subitem.spec.target_obj.kind.to_lowercase(),
        }
    };
    let target_obj_uri = {
        if subitem.spec.target_obj.namespace == "" {
            format!("api/{}/{}/{}", subitem.spec.target_obj.api_version,
                                        search_kind,
                                        subitem.spec.target_obj.name)
        } else {
            format!("api/{}/namespaces/{}/{}/{}", subitem.spec.target_obj.api_version,
                                                    subitem.spec.target_obj.namespace,
                                                    search_kind,
                                                    subitem.spec.target_obj.name)
        }
    };
    let full_get_url = format!("{}/{}", cluster.address, target_obj_uri);
    let target_current_contents = error_handler!(perform_get_request(&full_get_url, &cluster.token, &cluster.certificate), format!("Unable to get target contents for {}", target_obj_uri));
    return work_on_item_contents(target_current_contents, cluster, target_obj_uri, subitem.spec.patch);
}

fn work_on_item_contents(contents: String, cluster: Arc<Cluster>, target_uri: String, patch: String) -> Result<(), String> {
    let gen_obj: GenericItem = error_handler!(serde_json::from_str(&contents), format!("Unable to parse current object to generic object for {}", target_uri));
    let patch_conv: Value = error_handler!(serde_yaml::from_str(&patch), format!("Unable to parse patch from YAML for {}", target_uri));
    let patch_as_json_string: String = serde_json::json!(patch_conv).to_string();
    let full_get_url = format!("{}/{}", cluster.address, target_uri);
    let res = error_handler!(perform_patch_request(&full_get_url, &cluster.token, &cluster.certificate, patch_as_json_string), format!("Unable to execute patch request for {}", target_uri));
    let new_obj: GenericItem = error_handler!(serde_json::from_str(&res), format!("Unable to parse result to generic object for {}", target_uri));
    println!("Successfully patched {}: {} -> {}", target_uri, gen_obj.metadata.resource_version, new_obj.metadata.resource_version);
    Ok(())
}

/*
//
// FETCH API RESOURCES
//
*/

pub fn fetch_available_resources(cluster: Arc<Cluster>) -> Result<Arc<HashMap<String, String>>, String> {
    let rt = error_handler!(Runtime::new(), "Unable to start multithreading runtime for resources fetch");
    let full_get_url = format!("{}/{}", cluster.address, URL_LIST_ALL_AVAILABLE_RESOURCES);
    let api_groups = error_handler!(perform_get_request(&full_get_url, &cluster.token, &cluster.certificate), "Unable to get all available resources list");
    let mut agl: ApiGroupList = error_handler!(serde_json::from_str(&api_groups), "Unable to parse result into ApiGroupList".to_string());
    agl.groups.push(ApiGroupListItem {
                        name: "api".to_string(),
                        preferred_version: PreferredVersion {
                                                                group_version:"v1".to_string(),
                                                                version:"v1".to_string()
                                                            }
                                    }
                    );
    let mut all_to_wait_for = vec![];
    for ag in agl.groups {
        let ac = cluster.clone(); // cloning an Arc -> points to same heap space, only counter is increased
        all_to_wait_for.push(rt.spawn(async move {
            let suburl = {
                if ag.preferred_version.group_version == "v1" {
                    format!("{}/{}", "api", ag.preferred_version.group_version)
                } else {
                    format!("{}/{}", URL_LIST_ALL_AVAILABLE_RESOURCES, ag.preferred_version.group_version)
                }
            };
            let res = match get_api_group_data(suburl, ac) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Problem when parsing item: {}", e);
                    Box::new(vec![])
                },
            };
            res
        }));
    }
    let comb = rt.block_on(future::join_all(all_to_wait_for));
    let mut final_result = HashMap::new();
    for comb_item in comb {
        let item_list = match comb_item {
            Ok(il) => il,
            Err(e) => return Err(format!("Invalid API resource item returned: {}", e)),
        };
        for (item_kind, item_name) in item_list.iter() {
            final_result.insert(item_kind.clone(), item_name.clone());
        }
    }
    Ok(Arc::new(final_result))
}

fn get_api_group_data(gv: String, cluster: Arc<Cluster>) -> Result<Box<Vec<(String, String)>>, String> {
    let full_get_url = format!("{}/{}", cluster.address, gv);
    let api_group = error_handler!(perform_get_request(&full_get_url, &cluster.token, &cluster.certificate), "Unable to get API group data");
    let err_msg = format!("Unable to parse result into ApiResourceList ({})", api_group);
    let arl: ApiResourceList = error_handler!(serde_json::from_str(&api_group), err_msg.to_string());
    let mut result = vec![];
    for ar in arl.resources {
        if !ar.name.contains("/") {
            result.push((ar.kind, ar.name));
        }
    }
    Ok(Box::new(result))
}
