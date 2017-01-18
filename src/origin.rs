use package::{Package, PackageVersions};
use hyper::Client;
use hyper::header::{ContentType, Headers, Authorization, Bearer};
use hyper::status::StatusCode;
use habitat_core::package::{PackageArchive, PackageIdent};
use habitat_depot_client::PackageResults;
use habitat_depot_client::Client as DepotClient;
use habitat_common::ui::UI;
use rustc_serialize::{json, Decodable};

use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Origin {
    pub name: String,
    pub packages: Option<Vec<Package>>,
}

impl Origin {
    fn package_result_from_json<T: Decodable>(encoded: &str) -> PackageResults<T> {
        let results: PackageResults<T> = json::decode(&encoded).unwrap();
        results
    }

    pub fn get_package_list(&self, source: &String) -> Result<Vec<PackageIdent>, &'static str> {
        let client = Client::new();
        let mut range = 0;
        let mut packages: Vec<PackageIdent> = vec![];

        let all_packages = &vec![Package {
                                     name: "all".to_string(),
                                     versions: PackageVersions::All,
                                 }];
        let package_list: &Vec<Package> = match self.packages.as_ref() {
            Some(packages) => packages,
            None => all_packages,
        };

        for package in package_list {
            let mut range = 0;
            loop {
                let mut url = String::new();
                if package.name != "all" {
                    url = format!("{}/v1/depot/pkgs/{}/{}?range={}",
                                  source,
                                  self.name,
                                  package.name,
                                  range);
                } else {
                    url = format!("{}/v1/depot/pkgs/{}?range={}", source, self.name, range);
                }

                println!("Processing {}!", url);
                let mut response = match client.get(url.as_str()).send() {
                    Ok(response) => response,
                    Err(_) => {
                        panic!("Could not get packages for origin {} from source {}!",
                               self.name,
                               source)
                    }
                };

                let mut buffer = String::new();
                match response.read_to_string(&mut buffer) {
                    Ok(_) => (),
                    Err(_) => panic!("Could not read package list!"),
                };

                if buffer == "[]" {
                    break;
                }

                let mut results: PackageResults<PackageIdent> =
                    Origin::package_result_from_json(&buffer);
                let range_end = results.range_end;
                let total_count = results.total_count;
                packages.append(&mut results.package_list);
                if range_end < total_count {
                    range += 50;
                } else {
                    break;
                }
            }
        }
        Ok(packages)
    }

    pub fn download_packages(&self,
                             url: &String,
                             package_list: Vec<PackageIdent>,
                             tmp_dir: &str)
                             -> Vec<PackageArchive> {
        let source_url = format!("{}/v1/depot", url);
        let depot_client = DepotClient::new(source_url.as_str(), "hab", "0.15.0", None).unwrap();
        let mut archive_list: Vec<PackageArchive> = vec![];
        for package in package_list {
            println!("Downloading {}", &package);
            let archive = match depot_client.fetch_package(&package, &Path::new(tmp_dir), UI::default().progress()) {
                Ok(archive) => archive,
                Err(e) => panic!("{:?}", e),
            };
            archive_list.push(archive);
        }
        archive_list
    }

    pub fn upload_packages(&self,
                           url: &String,
                           archive_list: Vec<PackageArchive>,
                           tmp_dir: &str,
                           token: &str) {
        self.create_origin(url, token);
        let source_url = format!("{}/v1/depot", url);
        let depot_client = DepotClient::new(source_url.as_str(), "hab", "0.15.0", None).unwrap();
        for mut archive in archive_list {
            match depot_client.put_package(&mut archive, token, UI::default().progress()) {
                Ok(_) => println!("Uploaded the thing!"),
                Err(e) => println!("{}", e),
            }
        }
    }

    fn origin_exists(&self, url: &String) -> bool {
        let client = Client::new();
        let url = format!("{}/v1/depot/origins/{}", url, self.name);
        let mut response = client.get(url.as_str()).send().expect("Could not connect to Depot");

        match response.status {
            StatusCode::Ok => true,
            StatusCode::NotFound => false,
            _ => panic!("could not determine if origin exists or not"),
        }
    }

    pub fn create_origin(&self, url: &String, token: &str) {
        if self.origin_exists(url) {
            println!("Origin {} already exists on {}", self.name, url);
            return;
        }
        println!("Creating origin {} on {}", self.name, url);
        let client = Client::new();
        let url = format!("{}/v1/depot/origins", url);
        let payload = format!(r#"{{ "name": "{}" }}"#, self.name);
        let auth = Authorization(Bearer { token: token.to_string() });
        let mut response = client.post(url.as_str())
            .header(auth)
            .header(ContentType("application/x-www-form-encoded".parse().unwrap()))
            .body(payload.as_str())
            .send()
            .expect("Could not connect to Depot");

        self.download_keys(&url, "/tmp/habcache");
    }

    fn download_keys(&self, url: &String, tmp_dir: &str) {
        let depot_client = DepotClient::new(url.as_str(), "hab", "0.15.0", None).unwrap();
        match depot_client.show_origin_keys(self.name.as_str()) {
            Ok(keys) => {
                for key in keys {
                    depot_client.fetch_origin_key(self.name.as_str(),
                                          key.get_revision(),
                                          Path::new(tmp_dir),
                                          UI::default().progress())
                        .expect("Failed to get key!");
                }
            }
            Err(_) => panic!("Could not download origin keys!"),
        }
    }
}
