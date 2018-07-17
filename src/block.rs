use std::cell::RefCell;
use std::cmp::min;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::vec::Drain;

use common::constants::ZSTD_COMPRESSION_LEVEL;
use failure::Error;
use messenger::session::SESSION;

type BlockID = u64;

#[derive(Eq, PartialEq, Serialize, Deserialize)]
pub struct Block {
    id: BlockID,
    #[serde(skip_serializing, skip_deserializing)]
    data: Option<Vec<u8>>,
    url: Option<String>,
    used: u64,
    capacity: u64,
    dirty: bool,
}

impl Ord for Block {
    fn cmp(&self, other: &Block) -> Ordering {
        self.available().cmp(&other.available())
    }
}

impl PartialOrd for Block {
    fn partial_cmp(&self, other: &Block) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Block {
    fn new(id: BlockID, size: u64) -> Self {
        Self {
            id,
            used: 0,
            capacity: size,
            url: None,
            data: None,
            dirty: false,
        }
    }

    fn available(&self) -> u64 {
        self.capacity - self.used
    }

    pub fn data(&mut self) -> &mut Vec<u8> {
        match self.url.as_ref() {
            Some(url) => self.data.get_or_insert_with(|| {
                let mut data = Vec::new();
                SESSION
                    .lock()
                    .expect("Could not acquire Session lock")
                    .get_attachment(url, &mut data)
                    .expect("Could not page data block");
                zstd::decode_all(&data[..]).expect("Could not decode block")
            }),
            None => self.data.get_or_insert_with(Vec::new),
        }
    }

    fn fill(&mut self, data: &mut Drain<u8>) -> DataLoc {
        let offset = self.used;
        let available_size = self.available();
        let data_size = data.len() as u64;
        let write_size = min(available_size, data_size);
        self.data()
            .splice(offset as usize.., data.take(available_size as usize));
        self.used += data_size;
        self.dirty = true;
        DataLoc {
            block_id: self.id,
            offset,
            size: write_size,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DataLoc {
    pub block_id: u64,
    pub offset: u64,
    pub size: u64,
}

// Memory management
#[derive(Serialize, Deserialize)]
pub struct BlockPool {
    pub arena: RefCell<HashMap<BlockID, Block>>,
    max_num_blocks: u64,
    block_size: u64,
    block_id: BlockID,
}

impl BlockPool {
    pub fn new(max_num_blocks: u64, block_size: u64) -> Self {
        Self {
            arena: RefCell::new(HashMap::new()),
            max_num_blocks,
            block_size,
            block_id: 0,
        }
    }

    pub fn next_block_id(&mut self) -> BlockID {
        self.block_id += 1;
        self.block_id
    }

    pub fn create_block(&mut self) -> BlockID {
        let id = self.next_block_id();
        let block = Block::new(id, self.block_size);
        let mut arena = self.arena.borrow_mut();
        arena.insert(id, block);
        id
    }

    pub fn find(&mut self, size: u64) -> Vec<BlockID> {
        let mut remaining = size;
        let mut blocks = Vec::new();
        let block_size = self.block_size;
        while remaining > block_size {
            let block = self.create_block();
            remaining -= block_size;
            blocks.push(block);
        }
        if remaining > 0 {
            let (block_id, available) = {
                let arena = self.arena.borrow();
                let mut heap = BinaryHeap::from(arena.values().collect::<Vec<_>>());
                match heap.pop() {
                    Some(block) => (block.id, block.available()),
                    None => (0, 0),
                }
            };
            if available >= remaining {
                blocks.push(block_id);
            } else {
                blocks.push(self.create_block());
            }
        }
        blocks
    }

    pub fn alloc(&mut self, mut data: Vec<u8>) -> Vec<DataLoc> {
        let size = data.len() as u64;
        let blocks = self.find(size);
        let mut stream = data.drain(..);
        let arena = self.arena.get_mut();
        blocks
            .iter()
            .map(|block_id| {
                let block = arena.get_mut(block_id).unwrap();
                block.fill(&mut stream)
            })
            .collect()
    }

    pub fn sync(&mut self) -> Result<(), Error> {
        let mut arena = self.arena.borrow_mut();
        arena.iter_mut().for_each(|(_, block)| {
            if block.dirty {
                let mut session = SESSION.lock().expect("Could not acquire session lock");
                let thread_id = session.fbid.clone();
                let encoded =
                    zstd::encode_all(&block.data.as_ref().unwrap()[..], ZSTD_COMPRESSION_LEVEL)
                        .expect("Encoding failed")
                        .iter()
                        .map(|byte| *byte as char)
                        .collect();
                let resp = session
                    .attachment(encoded, thread_id)
                    .expect("Could not send attachment");
                let message = session
                    .get_message(resp.message_id)
                    .expect("Could not find message that was just sent");
                block.url = Some(message.attachments[0].url.clone());
                block.dirty = false;
            }
        });
        Ok(())
    }
}
