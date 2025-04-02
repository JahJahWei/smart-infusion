mod api_device;
mod api_bed;
mod api_patients;

pub use api_device::fetch_and_store_devices;
pub use api_bed::fetch_and_store_beds;
pub use api_patients::fetch_and_store_patients;
