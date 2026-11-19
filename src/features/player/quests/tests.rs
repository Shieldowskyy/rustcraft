//! Unit tests for questbook prerequisites, reward claims, and binary roundtrips.

use crate::features::blocks::BlockId;

use super::super::inventory::Inventory;
use super::tracker::count_item_in_inventory;
use super::QuestProgress;

#[test]
fn quest_progression_and_prerequisites() {
    let mut qp = QuestProgress::default();
    assert!(qp.is_unlocked(0));
    assert!(!qp.is_unlocked(1));

    for _ in 0..8 {
        qp.record_mined(BlockId::Wood);
    }
    assert!(qp.is_completed(0));
    assert!(qp.is_unlocked(1));
    assert_eq!(qp.claimable_count(), 1);

    let mut inv = Inventory::new();
    assert!(qp.claim_reward(0, &mut inv));
    assert!(qp.is_claimed(0));
    assert!(!qp.claim_reward(0, &mut inv));
    assert_eq!(qp.claimable_count(), 0);
    assert!(count_item_in_inventory(&inv, BlockId::Planks) >= 8);
}

#[test]
fn quest_progress_encode_decode_roundtrip() {
    let mut qp = QuestProgress::default();
    qp.record_mined(BlockId::Wood);
    qp.completed |= 1u64 << 0;
    qp.claimed |= 1u64 << 0;
    qp.progress[5] = 7;

    let encoded = qp.encode();
    assert_eq!(encoded.len(), QuestProgress::ENCODED_LEN);
    let decoded = QuestProgress::decode(&encoded).expect("decode should succeed");
    assert_eq!(qp, decoded);
}
