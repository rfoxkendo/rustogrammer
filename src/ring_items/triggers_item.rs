use crate::ring_items;
use std::time;
///
/// EventCountItems count the nunmber of triggers that have
/// been seen since the start of run.  This can be used for
/// determining the accepted event rate as well as, in a sampled client,
/// computing the fraction of events analyzed.
///

pub struct PhysicsEventCountItem {
    body_header: Option<ring_items::BodyHeader>,
    time_offset: u32,
    time_divisor: u32,
    absolute_time: time::SystemTime,
    original_sid: Option<u32>,
    event_count: u64,
}

impl PhysicsEventCountItem {
    pub fn new() -> PhysicsEventCountItem {
        PhysicsEventCountItem {
            body_header: None,
            time_offset: 0,
            time_divisor: 1,
            absolute_time: time::SystemTime::now(),
            original_sid: None,
            event_count: 0,
        }
    }
    pub fn get_bodyheader(&self) -> Option<ring_items::BodyHeader> {
        self.body_header
    }
    pub fn get_timeoffset(&self) -> u32 {
        self.time_offset
    }
    pub fn get_time_divisor(&self) -> u32 {
        self.time_divisor
    }
    pub fn get_offset_time(&self) -> f32 {
        (self.time_offset as f32) / (self.time_divisor as f32)
    }
    pub fn get_original_sid(&self) -> Option<u32> {
        self.original_sid
    }
    pub fn get_event_count(&self) -> u64 {
        self.event_count
    }
    pub fn get_absolute_time(&self) -> time::SystemTime {
        self.absolute_time
    }
    // Conversions:

    ///  Given a raw item if it is a ring_items::PHYSICS_EVENT_COUNT
    /// item make a PhysicsEventCountItem from it.

    pub fn from_raw(
        raw: &ring_items::RingItem,
        version: ring_items::RingVersion,
    ) -> Option<PhysicsEventCountItem> {
        if raw.type_id() == ring_items::PHYSICS_EVENT_COUNT {
            let mut result = Self::new();
            result.body_header = raw.get_bodyheader();
            let offset = if let Some(_) = result.body_header {
                ring_items::body_header_size()
            } else {
                0
            };
            let payload = raw.payload().as_slice();
            result.time_offset =
                u32::from_ne_bytes(payload[offset..offset + 4].try_into().unwrap());
            result.time_divisor =
                u32::from_ne_bytes(payload[offset + 4..offset + 8].try_into().unwrap());
            result.absolute_time = ring_items::raw_to_systime(
                u32::from_ne_bytes(payload[offset + 8..offset + 12].try_into().unwrap())
                    .try_into()
                    .unwrap(),
            );
            if version == ring_items::RingVersion::V11 {
                result.event_count =
                    u64::from_ne_bytes(payload[offset + 12..offset + 20].try_into().unwrap());
            } else {
                result.original_sid = Some(u32::from_ne_bytes(
                    payload[offset + 12..offset + 16].try_into().unwrap(),
                ));
                result.event_count =
                    u64::from_ne_bytes(payload[offset + 16..offset + 24].try_into().unwrap());
            }
            Some(result)
        } else {
            None
        }
    }
    /// Produce a raw ring item that is  the functional equivalent of
    /// self.
    ///
    pub fn to_raw(&self) -> ring_items::RingItem {
        let mut result = if let Some(bh) = self.body_header {
            ring_items::RingItem::new_with_body_header(
                ring_items::PHYSICS_EVENT_COUNT,
                bh.timestamp,
                bh.source_id,
                bh.barrier_type,
            )
        } else {
            ring_items::RingItem::new(ring_items::PHYSICS_EVENT_COUNT)
        };
        result
            .add(self.time_offset)
            .add(self.time_divisor)
            .add(ring_items::systime_to_raw(self.absolute_time));
        if let Some(sid) = self.original_sid {
            result.add(sid);
        }
        result.add(self.event_count);

        result
    }
}