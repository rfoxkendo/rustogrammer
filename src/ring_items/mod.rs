use std::fs::File;
use std::io::prelude::*;
use std::mem;

/// This is an raw ring item.   Raw in the
/// sense that the payload is just a soup of bytes.
/// However it wil have methods that allow conversion of this item
/// to more structured ring items based on the 'type' field.
///
pub struct RingItem {
    size: u32,
    type_id: u32,
    body_header_size: u32,
    payload: Vec<u8>,
}

pub enum RingItemError {
    HeaderReadFailed,
    InvalidHeader,
    FileTooSmall,
}
pub type RingItemResult = Result<RingItem, RingItemError>;

impl RingItem {
    // Private methods:

    // Read a u32:

    fn read_long(f: &mut File) -> Result<u32, u8> {
        let mut buf: [u8; 4] = [0; 4];

        if let Ok(_) = f.read_exact(&mut buf) {
            let long = u32::from_ne_bytes(buf);
            return Ok(long);
        }
        Err(0)
    }

    ///
    /// Create a new empty ring item of the given type.
    ///
    pub fn new(t: u32) -> RingItem {
        RingItem {
            size: 3 * mem::size_of::<u32>() as u32,
            type_id: t,
            body_header_size: mem::size_of::<u32>() as u32,
            payload: Vec::new(),
        }
    }
    /// create a new ring item with a 12.x body header.
    ///
    pub fn new_with_body_header(t: u32, stamp: u64, source: u32, barrier: u32) -> RingItem {
        let mut result = RingItem::new(t);
        result.body_header_size = (3 * mem::size_of::<u32>() + mem::size_of::<u64>()) as u32;

        result.add(stamp);
        result.add(source);
        result.add(barrier);

        result
    }

    pub fn size(&self) -> u32 {
        self.size
    }
    pub fn type_id(&self) -> u32 {
        self.type_id
    }
    pub fn has_body_header(&self) -> bool {
        self.body_header_size > mem::size_of::<u32>() as u32
    }
    ///  Add an object of type T to the ring buffer.  Note
    /// That the raw bytes are added therefore the item must
    /// not contain e.g. pointers.
    ///
    pub fn add<T>(&mut self, item: T) -> &mut RingItem {
        let pt = &item as *const T;
        let mut p = pt.cast::<u8>();

        // Now I have a byte pointer I can push the bytes of data
        // into the vector payload:

        for _i in 0..mem::size_of::<T>() {
            unsafe {
                self.payload.push(*p);
                p = p.offset(1);
            }
        }
        self.size = self.size + mem::size_of::<T>() as u32;
        self
    }
    /// Read a ring item from file.

    pub fn read_item(file: &mut File) -> RingItemResult {
        // Create a new ring item - type is unimportant since
        // it'll get overwitten.

        let mut item = RingItem::new(0);

        // The header fields must be read individually b/c
        // rust could have rearranged them  read only reads
        // to u8 arrays so we need to read and then copy into
        // the fields:

        if let Ok(n) = RingItem::read_long(file) {
            item.size = n;
        } else {
            return Err(RingItemError::HeaderReadFailed);
        }
        if item.size < 3 * mem::size_of::<u32>() as u32 {
            return Err(RingItemError::InvalidHeader);
        }

        if let Ok(n) = RingItem::read_long(file) {
            item.type_id = n;
        } else {
            return Err(RingItemError::HeaderReadFailed);
        }

        if let Ok(n) = RingItem::read_long(file) {
            item.body_header_size = n;
        } else {
            return Err(RingItemError::HeaderReadFailed);
        }

        // Figure out how many bytes are in the body
        // and read those into the veftor:

        let body_size: usize = (item.size as usize) - 3 * mem::size_of::<u32>();
        if body_size > 0 {
            item.payload.resize(body_size, 0);
            if let Err(_) = file.read_exact(item.payload.as_mut_slice()) {
                return Err(RingItemError::FileTooSmall);
            }
        }

        Ok(item)
    }
}
