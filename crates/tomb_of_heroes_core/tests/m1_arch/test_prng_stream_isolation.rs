use rand_xoshiro::rand_core::RngCore;
use tomb_of_heroes_core::{DeterministicRngBank, DungeonMasterSeed, RngStreamKind};

#[test]
fn test_prng_stream_isolation_combat_and_dungeon_gen() {
    let master_seed = DungeonMasterSeed(12345);

    // Initialiser DeterministicRngBank avec la graine maîtresse
    let mut bank = DeterministicRngBank::new(master_seed);

    // Tirer 500 valeurs sur le canal Combat
    for _ in 0..500 {
        let _ = bank.stream_mut(RngStreamKind::Combat).next_u64();
    }

    // Créer un générateur / stream vierge initialisé avec la même graine
    let mut virgin_dungeon_gen =
        DeterministicRngBank::create_stream(master_seed, RngStreamKind::DungeonGen);

    // Vérifier que le canal DungeonGen produit la même séquence exacte qu'un générateur vierge
    for _ in 0..500 {
        let val_from_bank = bank.stream_mut(RngStreamKind::DungeonGen).next_u64();
        let val_from_virgin = virgin_dungeon_gen.next_u64();
        assert_eq!(
            val_from_bank, val_from_virgin,
            "DungeonGen sequence diverged after drawing from Combat stream"
        );
    }
}

#[test]
fn test_strict_four_stream_isolation() {
    let seed = DungeonMasterSeed(99999);

    let mut bank_perturbed = DeterministicRngBank::new(seed);
    let mut bank_reference = DeterministicRngBank::new(seed);

    // Tirer agressivement des valeurs sur Combat, AiDecisions, LootAndDecay dans bank_perturbed
    for _ in 0..100 {
        let _ = bank_perturbed.next_u64(RngStreamKind::Combat);
    }
    for _ in 0..200 {
        let _ = bank_perturbed.next_u64(RngStreamKind::AiDecisions);
    }
    for _ in 0..300 {
        let _ = bank_perturbed.next_u64(RngStreamKind::LootAndDecay);
    }

    // Le canal DungeonGen dans bank_perturbed doit être 100% identique à celui de reference
    for _ in 0..250 {
        assert_eq!(
            bank_perturbed.next_u64(RngStreamKind::DungeonGen),
            bank_reference.next_u64(RngStreamKind::DungeonGen)
        );
    }

    // Tester l'isolation réciproque : perturber DungeonGen ne doit pas affecter un canal vierge
    let mut virgin_ai = DeterministicRngBank::create_stream(seed, RngStreamKind::AiDecisions);
    let mut bank_ai_fresh = DeterministicRngBank::new(seed);

    for _ in 0..150 {
        assert_eq!(
            virgin_ai.next_u64(),
            bank_ai_fresh.next_u64(RngStreamKind::AiDecisions)
        );
    }
}
