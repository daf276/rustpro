// What the diff harness reads from a card file: SUPPORT places cards with the card under test, CHECKS says what its
// scenarios must show besides diffing clean. Neither is part of the Lua.

// SUPPORT locations: one of these, or 0 for a random zone, with OPPONENT to place the card for the other player.
pub const HAND: u16 = 0x2;
pub const DECK: u16 = 0x1;
pub const MZONE: u16 = 0x4;
pub const SZONE: u16 = 0x8;
pub const GRAVE: u16 = 0x10;
pub const REMOVED: u16 = 0x20;
pub const EXTRA: u16 = 0x40;
pub const OPPONENT: u16 = 0x8000;

// Each check passes when its count over the card's own scenarios is above 0. A card without CHECKS has
// [Check::Resolves].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    // Some effect of the card resolves as a chain link.
    Resolves,
    // The effect with this description (aux::stringid) resolves.
    ResolvesEffect(u64),
    SpecialSummoned,
    // A card with this code, such as a token the card creates, is Special Summoned.
    SpecialSummonedCode(u32),
    // Moved as a Ritual Summon material.
    RitualMaterial,
    // Moved as a material from this location (HAND, MZONE, ...).
    MaterialFrom(u16),
    // A hint with this description is added to the card (MSG_CARD_HINT).
    CardHint(u64),
    // A message of this type about the card: the first card of MSG_EQUIP and MSG_CARD_TARGET, the card a message
    // names by code; a message about no card counts for the card under test.
    Message(u8),
    // Normal Summoned, or Set, with this many tributes.
    TributeSummoned(u32),
    TributeSet(u32),
    // Destroyed in place of another card (REASON_REPLACE).
    Replaced,
    // Counted and written out, never failing: for paths too rare to assert.
    Report(&'static Check),
}
