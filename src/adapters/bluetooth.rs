use serde::Serialize;

#[derive(Serialize, Debug)]
#[allow(non_snake_case)]
pub struct BleBeacon {
    pub macAddress: String,  
    pub signalStrength: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}