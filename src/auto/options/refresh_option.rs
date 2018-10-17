use partition_identity::PartitionID;
use std::fmt;

#[derive(Debug)]
pub struct RefreshOption {
    pub os_name:        String,
    pub os_pretty_name: String,
    pub os_version:     String,
    pub root_part:      String,
    pub home_part:      Option<String>,
    pub efi_part:       Option<String>,
    pub recovery_part:  Option<String>,
    pub can_retain_old: bool,
}

impl fmt::Display for RefreshOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let root_part: String = match PartitionID::new_uuid(self.root_part.clone()).get_device_path() {
            Some(uuid) => uuid.to_string_lossy().into(),
            None => "None".into(),
        };

        write!(f, "Refresh {} on {}", self.os_name, root_part)
    }
}
