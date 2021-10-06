use super::records::*;
use super::super::super::records::RecordsManager;
use super::super::super::super::buf_config::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub type BTreeNodeId = u64;
pub type BTreeCellId = u32;
pub type BtreeCellLoc = (BTreeNodeId, BTreeCellId);



pub struct NodeRecordPool {
    pub records: HashMap<u64, BNodeRecord>,
    pub records_manager: Rc<RefCell<RecordsManager>>,
}

impl NodeRecordPool {

    pub fn new(record_manager: Rc<RefCell<RecordsManager>>) -> Self {
        NodeRecordPool{ records: HashMap::new(), records_manager: record_manager }
    }

    pub fn is_empty_records_set(&mut self) -> bool {
        self.records_manager.borrow_mut().is_empty()
    }

    pub fn load_node_record_clone(&mut self, id: u64) -> Option<BNodeRecord> {
        if !self.records.contains_key(&id) {
            let mut data = [0u8; BTREE_NODE_RECORD_SIZE];
            self.records_manager.borrow_mut().load(id, &mut data).ok()?;
            self.records.insert(id, BNodeRecord::from_bytes(data));
        }
        Some(self.records.get(&id)?.clone())
    }

    pub fn load_node_record_ref(&mut self, id: u64) -> Option<&BNodeRecord> {
        if !self.records.contains_key(&id) {
            let mut data = [0u8; BTREE_NODE_RECORD_SIZE];
            self.records_manager.borrow_mut().load(id, &mut data).ok()?;
            self.records.insert(id, BNodeRecord::from_bytes(data));
        }
        Some(self.records.get(&id)?)
    }

    pub fn load_node_record_mut(&mut self, id: u64) -> Option<&mut BNodeRecord> {
        if !self.records.contains_key(&id) {
            let mut data = [0u8; BTREE_NODE_RECORD_SIZE];
            self.records_manager.borrow_mut().load(id, &mut data).ok()?;
            self.records.insert(id, BNodeRecord::from_bytes(data));
            
        }
        Some(self.records.get_mut(&id)?)
    }

    pub fn create_node_record(&mut self, node_record: BNodeRecord) -> Option<u64> {
        let id = self.records_manager.borrow_mut().create(&node_record.to_bytes()).ok()?;
        self.records.insert(id, node_record);
        Some(id)
    }

    pub fn save_all_node_records(&mut self) -> Option<()> {
        for r in &self.records {
            self.records_manager.borrow_mut().save(*r.0, &r.1.to_bytes()).ok()?
        }
        Some(())
    }

    pub fn free_cell_iter(&mut self) -> FreeCellIterator {
        FreeCellIterator { pool : self }
    }
}


pub struct FreeCellIterator<'a> {
    pool: &'a mut NodeRecordPool,
}

impl <'a> FreeCellIterator<'a> {
    fn load_or_create_free_cells_overflow_node(&mut self) -> Option<BTreeNodeId> {
        if self.pool.is_empty_records_set() {
            let mut first_free_node = BNodeRecord::new();
            first_free_node.set_overflow_node();
            let new_record = self.pool.create_node_record(first_free_node)?;
            self.set_first_free_list_node_ptr(new_record);
            Some(new_record)
        } else {
            let first_free_record_ptr = self.get_first_free_list_node_ptr();
            if first_free_record_ptr == 0 {
                let next_free_cells_overflow_node = self.create_overflow_node()?;
                self.set_first_free_list_node_ptr(next_free_cells_overflow_node);
                Some(next_free_cells_overflow_node)
            } else {
                let next_free_cell_node_ptr = if let Some(free_node_record) = self.pool.load_node_record_ref(first_free_record_ptr) {
                    if free_node_record.is_full() {
                        Some(free_node_record.next_free_cells_node_ptr)
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(next) = next_free_cell_node_ptr {
                    self.set_first_free_list_node_ptr(next);
                    self.load_or_create_free_cells_overflow_node()
                } else {
                    Some(first_free_record_ptr)
                }
            }
        }
    }

    
    fn create_overflow_node(&mut self) -> Option<BTreeNodeId> {
        let mut next_free_cells_overflow_node = BNodeRecord::new();
        next_free_cells_overflow_node.set_overflow_node();
        let id = self.pool.create_node_record(next_free_cells_overflow_node)?;
        Some(id)
    }

    fn get_first_free_list_node_ptr(&mut self) -> BTreeNodeId {
        let mut buf = [0u8; NODE_PTR_SIZE];
        buf.copy_from_slice(&self.pool.records_manager.borrow_mut().get_header_page_wrapper().get_header_payload_slice_ref()[NODE_PTR_SIZE..2*NODE_PTR_SIZE]);
        u64::from_be_bytes(buf)
    }

    fn set_first_free_list_node_ptr(&mut self, id: BTreeNodeId) {
        self.pool.records_manager.borrow_mut().get_header_page_wrapper().get_header_payload_slice_mut()[NODE_PTR_SIZE..2*NODE_PTR_SIZE].copy_from_slice(&id.to_be_bytes());
    }
}

impl <'a> Iterator for FreeCellIterator<'a> {
    type Item = BtreeCellLoc;
    fn next(&mut self) -> Option<Self::Item> {
        let free_cell_node_id = self.load_or_create_free_cells_overflow_node()?;
        if let Some(node_with_free_cells) = self.pool.load_node_record_mut(free_cell_node_id) {
            let mut cell_id = 0;
            for cell in &mut node_with_free_cells.cells {
                if !cell.is_active() {
                    return Some((free_cell_node_id, cell_id));
                }
                cell_id += 1;
            }
        }
        None
    }
}