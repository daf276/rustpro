use std::collections::BTreeMap;
#[cfg(feature = "cdb")]
use std::path::Path;

#[cfg(feature = "cdb")]
use rusqlite::{Connection, OpenFlags, Row};

pub const TYPE_MONSTER: u32 = 0x1;
pub const TYPE_SPELL: u32 = 0x2;
pub const TYPE_TRAP: u32 = 0x4;
pub const TYPE_NORMAL: u32 = 0x10;
pub const TYPE_FUSION: u32 = 0x40;
pub const TYPE_RITUAL: u32 = 0x80;
pub const TYPE_TRAPMONSTER: u32 = 0x100;
pub const TYPE_SYNCHRO: u32 = 0x2000;
pub const TYPE_TOKEN: u32 = 0x4000;
pub const TYPE_FIELD: u32 = 0x8_0000;
pub const TYPE_XYZ: u32 = 0x80_0000;
pub const TYPE_PENDULUM: u32 = 0x100_0000;
pub const TYPE_SPSUMMON: u32 = 0x200_0000;
pub const TYPE_LINK: u32 = 0x400_0000;
pub const TYPE_EXTRA: u32 = TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK;

pub const OPCODE_ADD: u64 = 0x4000_0000_0000_0000;
pub const OPCODE_SUB: u64 = 0x4000_0001_0000_0000;
pub const OPCODE_MUL: u64 = 0x4000_0002_0000_0000;
pub const OPCODE_DIV: u64 = 0x4000_0003_0000_0000;
pub const OPCODE_AND: u64 = 0x4000_0004_0000_0000;
pub const OPCODE_OR: u64 = 0x4000_0005_0000_0000;
pub const OPCODE_NEG: u64 = 0x4000_0006_0000_0000;
pub const OPCODE_NOT: u64 = 0x4000_0007_0000_0000;
pub const OPCODE_BAND: u64 = 0x4000_0008_0000_0000;
pub const OPCODE_BOR: u64 = 0x4000_0009_0000_0000;
pub const OPCODE_BNOT: u64 = 0x4000_0010_0000_0000;
pub const OPCODE_BXOR: u64 = 0x4000_0011_0000_0000;
pub const OPCODE_LSHIFT: u64 = 0x4000_0012_0000_0000;
pub const OPCODE_RSHIFT: u64 = 0x4000_0013_0000_0000;
pub const OPCODE_ALLOW_ALIASES: u64 = 0x4000_0014_0000_0000;
pub const OPCODE_ALLOW_TOKENS: u64 = 0x4000_0015_0000_0000;
pub const OPCODE_ISCODE: u64 = 0x4000_0100_0000_0000;
pub const OPCODE_ISSETCARD: u64 = 0x4000_0101_0000_0000;
pub const OPCODE_ISTYPE: u64 = 0x4000_0102_0000_0000;
pub const OPCODE_ISRACE: u64 = 0x4000_0103_0000_0000;
pub const OPCODE_ISATTRIBUTE: u64 = 0x4000_0104_0000_0000;
pub const OPCODE_GETCODE: u64 = 0x4000_0105_0000_0000;
pub const OPCODE_GETTYPE: u64 = 0x4000_0107_0000_0000;
pub const OPCODE_GETRACE: u64 = 0x4000_0108_0000_0000;
pub const OPCODE_GETATTRIBUTE: u64 = 0x4000_0109_0000_0000;

#[derive(Debug, thiserror::Error)]
pub enum CardsError {
    #[error("cannot list card databases: {0}")]
    Directory(#[from] std::io::Error),
    #[cfg(feature = "cdb")]
    #[error("cannot read card database: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Card {
    pub code: u32,
    pub alias: u32,
    pub ot: u32,
    pub card_type: u32,
    pub level: u32,
    pub lscale: u32,
    pub rscale: u32,
    pub attribute: u32,
    pub race: u64,
    pub attack: i32,
    pub defense: i32,
    pub link_marker: u32,
    // Zero-terminated as OCG_CardData expects, so the last entry is always 0.
    pub setcodes: [u16; 5],
}

impl Card {
    #[cfg(feature = "cdb")]
    // cdb packing: setcode holds four u16 archetypes, level holds the pendulum scales in its upper bytes,
    // and a Link monster's def column holds its link markers.
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let card_type = row.get("type")?;
        let level: u32 = row.get("level")?;
        let defense = row.get("def")?;
        let setcode = row.get::<_, i64>("setcode")? as u64;
        let is_link = card_type & TYPE_LINK != 0;
        let mut setcodes = [0; 5];
        let codes = (0..4).map(|index| (setcode >> (16 * index)) as u16).filter(|&code| code != 0);
        setcodes.iter_mut().zip(codes).for_each(|(slot, code)| *slot = code);
        Ok(Self {
            code: row.get("id")?,
            alias: row.get("alias")?,
            ot: row.get("ot")?,
            card_type,
            level: level & 0xff,
            lscale: (level >> 24) & 0xff,
            rscale: (level >> 16) & 0xff,
            attribute: row.get("attribute")?,
            race: row.get::<_, i64>("race")? as u64,
            attack: row.get("atk")?,
            defense: if is_link { 0 } else { defense },
            link_marker: if is_link { defense as u32 } else { 0 },
            setcodes,
        })
    }

    pub fn is_alternate_art(&self) -> bool {
        self.alias != 0 && self.code.abs_diff(self.alias) < 10
    }

    // Port of `is_declarable` in playerop.cpp: a stack machine over the opcodes of MSG_ANNOUNCE_CARD.
    pub fn is_declarable(&self, opcodes: &[u64]) -> bool {
        const MARINE_DOLPHIN: u32 = 78734254;
        const TWINKLE_MOSS: u32 = 13857930;
        let mut stack: Vec<i64> = Vec::new();
        let (mut aliases, mut tokens) = (false, false);
        for &opcode in opcodes {
            let value = match opcode {
                OPCODE_ADD => binary(&mut stack, i64::wrapping_add),
                OPCODE_SUB => binary(&mut stack, i64::wrapping_sub),
                OPCODE_MUL => binary(&mut stack, i64::wrapping_mul),
                OPCODE_DIV => binary(&mut stack, |left, right| left.checked_div(right).unwrap_or_default()),
                OPCODE_AND => binary(&mut stack, |left, right| i64::from(left != 0 && right != 0)),
                OPCODE_OR => binary(&mut stack, |left, right| i64::from(left != 0 || right != 0)),
                OPCODE_NEG => stack.pop().map(i64::wrapping_neg),
                OPCODE_NOT => stack.pop().map(|value| i64::from(value == 0)),
                OPCODE_BAND => binary(&mut stack, |left, right| left & right),
                OPCODE_BOR => binary(&mut stack, |left, right| left | right),
                OPCODE_BXOR => binary(&mut stack, |left, right| left ^ right),
                OPCODE_BNOT => stack.pop().map(|value| !value),
                OPCODE_LSHIFT => binary(&mut stack, |left, right| left.wrapping_shl(right as u32)),
                OPCODE_RSHIFT => binary(&mut stack, |left, right| left.wrapping_shr(right as u32)),
                OPCODE_ISCODE => stack.pop().map(|value| i64::from(self.code == value as u32)),
                OPCODE_ISTYPE => stack.pop().map(|value| i64::from(self.card_type) & value),
                OPCODE_ISRACE => stack.pop().map(|value| (self.race & value as u64) as i64),
                OPCODE_ISATTRIBUTE => stack.pop().map(|value| i64::from(self.attribute) & value),
                OPCODE_ISSETCARD => stack.pop().map(|value| i64::from(self.is_set_card(value as i32))),
                OPCODE_GETCODE => Some(i64::from(self.code)),
                OPCODE_GETTYPE => Some(i64::from(self.card_type)),
                OPCODE_GETRACE => Some(self.race as i64),
                OPCODE_GETATTRIBUTE => Some(i64::from(self.attribute)),
                OPCODE_ALLOW_ALIASES => {
                    aliases = true;
                    None
                }
                OPCODE_ALLOW_TOKENS => {
                    tokens = true;
                    None
                }
                literal => Some(literal as i64),
            };
            stack.extend(value);
        }
        let token_monster = TYPE_MONSTER | TYPE_TOKEN;
        matches!(stack.as_slice(), [result] if *result != 0)
            && (self.code == MARINE_DOLPHIN
                || self.code == TWINKLE_MOSS
                || ((aliases || self.alias == 0) && (tokens || self.card_type & token_monster != token_monster)))
    }

    fn is_set_card(&self, set_code: i32) -> bool {
        let (archetype, subtype) = ((set_code & 0xfff) as u16, (set_code & 0xf000) as u16);
        self.setcodes
            .iter()
            .any(|&code| code != 0 && code & 0xfff == archetype && code & 0xf000 & subtype == subtype)
    }
}

#[derive(Debug, Default)]
pub struct CardDatabase {
    cards: BTreeMap<u32, Card>,
}

impl FromIterator<Card> for CardDatabase {
    fn from_iter<I: IntoIterator<Item = Card>>(cards: I) -> Self {
        Self {
            cards: cards.into_iter().map(|card| (card.code, card)).collect(),
        }
    }
}

impl CardDatabase {
    #[cfg(feature = "cdb")]
    pub fn load(directory: &Path) -> Result<Self, CardsError> {
        let mut paths = Vec::new();
        for entry in std::fs::read_dir(directory)? {
            let path = entry?.path();
            let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
            if name == "cards.cdb" || (name.starts_with("release-") && name.ends_with(".cdb")) {
                paths.push(path);
            }
        }
        paths.sort();
        let mut cards = BTreeMap::new();
        for path in paths {
            let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            let mut statement = connection.prepare("SELECT * FROM datas")?;
            for card in statement.query_map([], Card::from_row)? {
                let card = card?;
                cards.insert(card.code, card);
            }
        }
        Ok(Self { cards })
    }

    pub fn get(&self, code: u32) -> Option<&Card> {
        self.cards.get(&code)
    }

    pub fn cards(&self) -> impl Iterator<Item = &Card> {
        self.cards.values()
    }
}

// Operators with too few operands push nothing, as in the core.
fn binary(stack: &mut Vec<i64>, operator: impl Fn(i64, i64) -> i64) -> Option<i64> {
    let right = stack.pop()?;
    let Some(left) = stack.pop() else {
        stack.push(right);
        return None;
    };
    Some(operator(left, right))
}
