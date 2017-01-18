#[derive(Serialize, Deserialize)]
pub enum PackageVersions {
    All,
    Latest,
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub versions: PackageVersions,
}
