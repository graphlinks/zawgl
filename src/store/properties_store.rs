use super::records::*;
use super::store::*;

pub struct PropertiesStore {
    prop_records_store: Store,
}

impl PropertiesStore {
    pub fn new(file: &str) -> Self {
        PropertiesStore {prop_records_store: Store::new(file, 42)}
    }
    pub fn save(&mut self, pr: DynamicStoreRecord) {
        self.prop_records_store.save(&pr_to_bytes(pr));
    }
    pub fn load(&mut self, pr_id: u64) -> DynamicStoreRecord {
        let mut data: [u8; 42] = [0; 42];
        self.prop_records_store.load(pr_id, &mut data);
        pr_from_bytes(data)
    }
}