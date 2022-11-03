use super::{
    BaseRocksSecondaryIndex, ColumnFamilyName, IdRow, IndexId, MetaStoreEvent, QueueItem,
    QueueItemStatus, RocksSecondaryIndex, RocksTable, TableId,
};
use crate::{base_rocks_secondary_index, rocks_table_impl};

use chrono::{DateTime, Utc};
use rocksdb::DB;
use serde::{Deserialize, Deserializer};

impl QueueItem {
    pub fn new(key: String, value: String, status: QueueItemStatus, priority: i64) -> Self {
        QueueItem {
            key,
            value,
            status,
            priority,
            created: Utc::now(),
            heartbeat: None,
        }
    }

    pub fn get_key(&self) -> &String {
        &self.key
    }

    pub fn get_value(&self) -> &String {
        &self.value
    }

    pub fn get_status(&self) -> &QueueItemStatus {
        &self.status
    }

    pub fn get_heartbeat(&self) -> &Option<DateTime<Utc>> {
        &self.heartbeat
    }

    pub fn get_created(&self) -> &DateTime<Utc> {
        &self.created
    }

    pub fn status_default() -> QueueItemStatus {
        QueueItemStatus::Pending
    }

    pub fn update_heartbeat(&self) -> Self {
        let mut new = self.clone();
        new.heartbeat = Some(Utc::now());

        new
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum QueueItemRocksIndex {
    ByKey = 1,
    ByStatus = 2,
}

rocks_table_impl!(
    QueueItem,
    QueueItemRocksTable,
    TableId::QueueItems,
    {
        vec![
            Box::new(QueueItemRocksIndex::ByKey),
            Box::new(QueueItemRocksIndex::ByStatus),
        ]
    },
    ColumnFamilyName::Cache
);

#[derive(Hash, Clone, Debug)]
pub enum QueueItemIndexKey {
    ByKey(String),
    ByStatus(QueueItemStatus),
}

base_rocks_secondary_index!(QueueItem, QueueItemRocksIndex);

impl RocksSecondaryIndex<QueueItem, QueueItemIndexKey> for QueueItemRocksIndex {
    fn typed_key_by(&self, row: &QueueItem) -> QueueItemIndexKey {
        match self {
            QueueItemRocksIndex::ByKey => QueueItemIndexKey::ByKey(row.get_key().clone()),
            QueueItemRocksIndex::ByStatus => QueueItemIndexKey::ByStatus(row.get_status().clone()),
        }
    }

    fn key_to_bytes(&self, key: &QueueItemIndexKey) -> Vec<u8> {
        match key {
            QueueItemIndexKey::ByKey(s) => s.as_bytes().to_vec(),
            QueueItemIndexKey::ByStatus(s) => {
                let mut r = Vec::with_capacity(1);

                match s {
                    QueueItemStatus::Pending => r.push(0_u8),
                    QueueItemStatus::Active => r.push(1_u8),
                    QueueItemStatus::Finished => r.push(2_u8),
                }

                r
            }
        }
    }

    fn is_unique(&self) -> bool {
        match self {
            QueueItemRocksIndex::ByKey => true,
            QueueItemRocksIndex::ByStatus => false,
        }
    }

    fn version(&self) -> u32 {
        match self {
            QueueItemRocksIndex::ByKey => 1,
            QueueItemRocksIndex::ByStatus => 1,
        }
    }

    fn is_ttl(&self) -> bool {
        false
    }

    fn get_id(&self) -> IndexId {
        *self as IndexId
    }
}
