// MIT License
// Copyright (c) 2022 Alexandre RICCIARDI
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use super::{MutableRecordsManager, records::*};
use super::super::super::super::buf_config::*;
use std::collections::HashMap;

pub type BTreeNodeId = u64;
pub type BTreeCellId = u32;
pub type BtreeCellLoc = (BTreeNodeId, BTreeCellId);



pub struct NodeRecordPool {
    pub records: HashMap<u64, BNodeRecord>,
    pub records_manager: MutableRecordsManager,
}

impl NodeRecordPool {

    pub fn new(record_manager: MutableRecordsManager) -> Self {
        NodeRecordPool{ records: HashMap::new(), records_manager: record_manager }
    }

    pub fn is_empty_records_set(&mut self) -> bool {
        self.records_manager.lock().unwrap().is_empty()
    }

    pub fn load_node_record_clone(&mut self, id: u64) -> Option<BNodeRecord> {
        if !self.records.contains_key(&id) {
            let mut data = [0u8; BTREE_NODE_RECORD_SIZE];
            self.records_manager.lock().unwrap().load(id, &mut data).ok()?;
            self.records.insert(id, BNodeRecord::from_bytes(data));
        }
        Some(self.records.get(&id)?.clone())
    }

    pub fn load_node_record_ref(&mut self, id: u64) -> Option<&BNodeRecord> {
        if !self.records.contains_key(&id) {
            let mut data = [0u8; BTREE_NODE_RECORD_SIZE];
            self.records_manager.lock().unwrap().load(id, &mut data).ok()?;
            self.records.insert(id, BNodeRecord::from_bytes(data));
        }
        Some(self.records.get(&id)?)
    }

    pub fn load_node_record_mut(&mut self, id: u64) -> Option<&mut BNodeRecord> {
        if !self.records.contains_key(&id) {
            let mut data = [0u8; BTREE_NODE_RECORD_SIZE];
            self.records_manager.lock().unwrap().load(id, &mut data).ok()?;
            self.records.insert(id, BNodeRecord::from_bytes(data));
            
        }
        Some(self.records.get_mut(&id)?)
    }

    pub fn create_node_record(&mut self, node_record: BNodeRecord) -> Option<u64> {
        let id = self.records_manager.lock().unwrap().create(&node_record.to_bytes()).ok()?;
        self.records.insert(id, node_record);
        Some(id)
    }

    pub fn save_all_node_records(&mut self) -> Option<()> {
        for r in &self.records {
            self.records_manager.lock().unwrap().save(*r.0, &r.1.to_bytes()).ok()?
        }
        Some(())
    }

    pub fn free_cell_iter(&mut self) -> FreeCellIterator {
        FreeCellIterator { pool : self }
    }

    pub fn insert_cell_in_free_slot(&mut self, cell_record: &CellRecord) -> Option<BtreeCellLoc> {
        let mut iter = self.free_cell_iter();
        let next_free_cell_loc = iter.next()?;
        let mut nr = self.load_node_record_mut(next_free_cell_loc.0)?;
        nr.cells[next_free_cell_loc.1 as usize] = *cell_record;
        Some(next_free_cell_loc)
    }

    pub fn disable_cell_records(&mut self, root_cell_record_loc: BtreeCellLoc) -> Option<()> {
        let mut next_cell_loc = root_cell_record_loc;
        while next_cell_loc.0 != 0 {
            let nr = self.load_node_record_mut(next_cell_loc.0)?;
            let mut curr_cell = nr.cells[next_cell_loc.1 as usize];
            curr_cell.set_inactive();
            next_cell_loc = curr_cell.get_next_cell_location();
        }
        Some(())
    }

    pub fn append_node_record_to_free_list(&mut self, node_record_id: BTreeNodeId, node_record: &mut BNodeRecord) {
        node_record.next_free_cells_node_ptr = self.get_first_free_list_node_ptr();
        self.set_first_free_list_node_ptr(node_record_id);
    }
    
    fn get_first_free_list_node_ptr(&mut self) -> BTreeNodeId {
        let mut buf = [0u8; NODE_PTR_SIZE];
        buf.copy_from_slice(&self.records_manager.lock().unwrap().get_header_page_wrapper().get_header_payload_slice_ref()[NODE_PTR_SIZE..2*NODE_PTR_SIZE]);
        u64::from_be_bytes(buf)
    }

    fn set_first_free_list_node_ptr(&mut self, id: BTreeNodeId) {
        self.records_manager.lock().unwrap().get_header_page_wrapper().get_header_payload_slice_mut()[NODE_PTR_SIZE..2*NODE_PTR_SIZE].copy_from_slice(&id.to_be_bytes());
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
        buf.copy_from_slice(&self.pool.records_manager.lock().unwrap().get_header_page_wrapper().get_header_payload_slice_ref()[NODE_PTR_SIZE..2*NODE_PTR_SIZE]);
        u64::from_be_bytes(buf)
    }

    fn set_first_free_list_node_ptr(&mut self, id: BTreeNodeId) {
        self.pool.records_manager.lock().unwrap().get_header_page_wrapper().get_header_payload_slice_mut()[NODE_PTR_SIZE..2*NODE_PTR_SIZE].copy_from_slice(&id.to_be_bytes());
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