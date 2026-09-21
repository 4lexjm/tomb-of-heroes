use std::collections::HashSet;
use tomb_of_heroes_core::{LogicId, LogicIdError, LogicIdGenerator};

#[test]
fn test_logic_id_sequential_allocation_10k() {
    let mut generator = LogicIdGenerator::new();
    let mut seen_ids = HashSet::with_capacity(10_000);
    let mut prev_id_val = 0u64;

    for i in 1..=10_000 {
        let alloc_res = generator.allocate();
        assert!(alloc_res.is_ok());

        if let Ok(id) = alloc_res {
            if i == 1 {
                assert_eq!(id, LogicId(1));
                assert_eq!(id, LogicId::FIRST);
            } else {
                assert_eq!(id.as_u64(), prev_id_val + 1);
            }

            assert!(seen_ids.insert(id), "Duplicate LogicId detected: {:?}", id);
            prev_id_val = id.as_u64();
        }
    }

    assert_eq!(seen_ids.len(), 10_000);
}

#[test]
fn test_logic_id_overflow_at_u64_max() {
    let mut generator = LogicIdGenerator::from_raw(u64::MAX);

    // If generator is at u64::MAX, allocation must return Err(LogicIdError::Overflow)
    let result = generator.allocate();
    assert_eq!(result, Err(LogicIdError::Overflow));

    // Calling next_id alias should also fail safely without panicking
    let next_result = generator.next_id();
    assert_eq!(next_result, Err(LogicIdError::Overflow));
}

#[test]
fn test_logic_id_display_and_serde() {
    let id = LogicId(42);
    assert_eq!(format!("{id}"), "LogicId(42)");

    let serialized = serde_json::to_string(&id);
    assert!(serialized.is_ok());
    if let Ok(json) = serialized {
        let deserialized: Result<LogicId, _> = serde_json::from_str(&json);
        assert_eq!(deserialized.ok(), Some(id));
    }
}
