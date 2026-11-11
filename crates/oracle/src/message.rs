use ocgcore_sys::{
    MSG_ANNOUNCE_ATTRIB, MSG_ANNOUNCE_CARD, MSG_ANNOUNCE_NUMBER, MSG_ANNOUNCE_RACE, MSG_NEW_TURN, MSG_RETRY, MSG_ROCK_PAPER_SCISSORS,
    MSG_SELECT_BATTLECMD, MSG_SELECT_CARD, MSG_SELECT_CHAIN, MSG_SELECT_COUNTER, MSG_SELECT_DISFIELD, MSG_SELECT_EFFECTYN,
    MSG_SELECT_IDLECMD, MSG_SELECT_OPTION, MSG_SELECT_PLACE, MSG_SELECT_POSITION, MSG_SELECT_SUM, MSG_SELECT_TRIBUTE,
    MSG_SELECT_UNSELECT_CARD, MSG_SELECT_YESNO, MSG_SORT_CARD, MSG_SORT_CHAIN, MSG_WIN,
};

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("empty message")]
    Empty,
    #[error("message {0} is truncated")]
    Truncated(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Retry,
    Win { player: u8, reason: u8 },
    NewTurn { player: u8 },
    Prompt(Prompt),
    Other(u8),
}

// Card lists are reduced to what bounds a valid response; the codes and locations stay in the raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prompt {
    BattleCommand {
        player: u8,
        activations: usize,
        attacks: usize,
        to_main2: bool,
        to_end: bool,
    },
    IdleCommand {
        player: u8,
        summons: usize,
        special_summons: usize,
        repositions: usize,
        monster_sets: usize,
        spell_sets: usize,
        activations: usize,
        to_battle: bool,
        to_end: bool,
        shuffle: bool,
    },
    EffectYesNo {
        player: u8,
    },
    YesNo {
        player: u8,
    },
    Option {
        player: u8,
        options: Vec<u64>,
    },
    Card {
        player: u8,
        cancelable: bool,
        min: u32,
        max: u32,
        count: usize,
    },
    Chain {
        player: u8,
        forced: bool,
        count: usize,
    },
    // Also MSG_SELECT_DISFIELD. `blocked` bits: own monster zones 0-6, own spell zones 8-15, the opponent's at +16.
    Place {
        player: u8,
        count: u8,
        blocked: u32,
    },
    Position {
        player: u8,
        positions: u8,
    },
    Tribute {
        player: u8,
        cancelable: bool,
        min: u32,
        max: u32,
        releases: Vec<u8>,
    },
    // Also MSG_SORT_CHAIN.
    Sort {
        player: u8,
        count: usize,
    },
    Counter {
        player: u8,
        count: u16,
        counters: Vec<u16>,
    },
    // Each param holds a value in its low and an optional alternative in its high u16.
    Sum {
        player: u8,
        overflow: bool,
        target: u32,
        min: u32,
        max: u32,
        must: Vec<u32>,
        params: Vec<u32>,
    },
    UnselectCard {
        player: u8,
        finishable: bool,
        cancelable: bool,
        min: u32,
        max: u32,
        count: usize,
    },
    RockPaperScissors {
        player: u8,
    },
    AnnounceRace {
        player: u8,
        count: u8,
        available: u64,
    },
    AnnounceAttribute {
        player: u8,
        count: u8,
        available: u32,
    },
    AnnounceCard {
        player: u8,
        opcodes: Vec<u64>,
    },
    AnnounceNumber {
        player: u8,
        options: Vec<u64>,
    },
}

// The core writes fields with memcpy, so they are packed and in native byte order.
pub fn parse(message: &[u8]) -> Result<Message, MessageError> {
    let (&id, body) = message.split_first().ok_or(MessageError::Empty)?;
    let reader = &mut Reader { id, bytes: body };
    Ok(match id {
        MSG_RETRY => Message::Retry,
        MSG_WIN => Message::Win {
            player: reader.u8()?,
            reason: reader.u8()?,
        },
        MSG_NEW_TURN => Message::NewTurn { player: reader.u8()? },
        _ => prompt(reader)?.map_or(Message::Other(id), Message::Prompt),
    })
}

// Card entry sizes: code u32, then controller/location u8 and sequence u32 or u8, or a 10-byte loc_info.
const CARD: usize = 10;
const CARD_SHORT: usize = 7;
const CARD_INFO: usize = 14;
const CHAIN_ENTRY: usize = CARD + 8 + 1;

fn prompt(reader: &mut Reader) -> Result<Option<Prompt>, MessageError> {
    // Every prompt starts with the player; other messages may have an empty body.
    let Some(&player) = reader.bytes.first() else { return Ok(None) };
    reader.skip(1)?;
    Ok(Some(match reader.id {
        MSG_SELECT_BATTLECMD => Prompt::BattleCommand {
            player,
            activations: reader.skip_list(CHAIN_ENTRY)?,
            attacks: reader.skip_list(8)?,
            to_main2: reader.flag()?,
            to_end: reader.flag()?,
        },
        MSG_SELECT_IDLECMD => Prompt::IdleCommand {
            player,
            summons: reader.skip_list(CARD)?,
            special_summons: reader.skip_list(CARD)?,
            repositions: reader.skip_list(CARD_SHORT)?,
            monster_sets: reader.skip_list(CARD)?,
            spell_sets: reader.skip_list(CARD)?,
            activations: reader.skip_list(CHAIN_ENTRY)?,
            to_battle: reader.flag()?,
            to_end: reader.flag()?,
            shuffle: reader.flag()?,
        },
        MSG_SELECT_EFFECTYN => Prompt::EffectYesNo { player },
        MSG_SELECT_YESNO => Prompt::YesNo { player },
        MSG_SELECT_OPTION => Prompt::Option {
            player,
            options: reader.options()?,
        },
        MSG_SELECT_CARD => Prompt::Card {
            player,
            cancelable: reader.flag()?,
            min: reader.u32()?,
            max: reader.u32()?,
            count: reader.skip_list(CARD_INFO)?,
        },
        MSG_SELECT_CHAIN => {
            reader.skip(1)?;
            let forced = reader.flag()?;
            reader.skip(8)?;
            Prompt::Chain {
                player,
                forced,
                count: reader.skip_list(CARD_INFO + 8 + 1)?,
            }
        }
        MSG_SELECT_PLACE | MSG_SELECT_DISFIELD => Prompt::Place {
            player,
            count: reader.u8()?,
            blocked: reader.u32()?,
        },
        MSG_SELECT_POSITION => {
            reader.skip(4)?;
            Prompt::Position {
                player,
                positions: reader.u8()?,
            }
        }
        MSG_SELECT_TRIBUTE => Prompt::Tribute {
            player,
            cancelable: reader.flag()?,
            min: reader.u32()?,
            max: reader.u32()?,
            releases: reader.list(|reader| reader.skip(CARD).and_then(|()| reader.u8()))?,
        },
        MSG_SORT_CARD | MSG_SORT_CHAIN => Prompt::Sort {
            player,
            count: reader.skip_list(13)?,
        },
        MSG_SELECT_COUNTER => {
            reader.skip(2)?;
            let count = reader.u16()?;
            Prompt::Counter {
                player,
                count,
                counters: reader.list(|reader| reader.skip(CARD_SHORT).and_then(|()| reader.u16()))?,
            }
        }
        MSG_SELECT_SUM => Prompt::Sum {
            player,
            overflow: reader.flag()?,
            target: reader.u32()?,
            min: reader.u32()?,
            max: reader.u32()?,
            must: reader.list(|reader| reader.skip(CARD_INFO).and_then(|()| reader.u32()))?,
            params: reader.list(|reader| reader.skip(CARD_INFO).and_then(|()| reader.u32()))?,
        },
        MSG_SELECT_UNSELECT_CARD => Prompt::UnselectCard {
            player,
            finishable: reader.flag()?,
            cancelable: reader.flag()?,
            min: reader.u32()?,
            max: reader.u32()?,
            count: reader.skip_list(CARD_INFO)? + reader.skip_list(CARD_INFO)?,
        },
        MSG_ROCK_PAPER_SCISSORS => Prompt::RockPaperScissors { player },
        MSG_ANNOUNCE_RACE => Prompt::AnnounceRace {
            player,
            count: reader.u8()?,
            available: reader.u64()?,
        },
        MSG_ANNOUNCE_ATTRIB => Prompt::AnnounceAttribute {
            player,
            count: reader.u8()?,
            available: reader.u32()?,
        },
        MSG_ANNOUNCE_CARD => Prompt::AnnounceCard {
            player,
            opcodes: reader.options()?,
        },
        MSG_ANNOUNCE_NUMBER => Prompt::AnnounceNumber {
            player,
            options: reader.options()?,
        },
        _ => return Ok(None),
    }))
}

struct Reader<'a> {
    id: u8,
    bytes: &'a [u8],
}

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], MessageError> {
        let (head, rest) = self.bytes.split_first_chunk::<N>().ok_or(MessageError::Truncated(self.id))?;
        self.bytes = rest;
        Ok(*head)
    }

    fn skip(&mut self, length: usize) -> Result<(), MessageError> {
        self.bytes = self.bytes.get(length..).ok_or(MessageError::Truncated(self.id))?;
        Ok(())
    }

    fn u8(&mut self) -> Result<u8, MessageError> {
        self.take().map(u8::from_ne_bytes)
    }

    fn flag(&mut self) -> Result<bool, MessageError> {
        self.u8().map(|value| value != 0)
    }

    fn u16(&mut self) -> Result<u16, MessageError> {
        self.take().map(u16::from_ne_bytes)
    }

    fn u32(&mut self) -> Result<u32, MessageError> {
        self.take().map(u32::from_ne_bytes)
    }

    fn u64(&mut self) -> Result<u64, MessageError> {
        self.take().map(u64::from_ne_bytes)
    }

    fn list<T>(&mut self, mut item: impl FnMut(&mut Self) -> Result<T, MessageError>) -> Result<Vec<T>, MessageError> {
        (0..self.u32()?).map(|_| item(self)).collect()
    }

    fn skip_list(&mut self, stride: usize) -> Result<usize, MessageError> {
        self.list(|reader| reader.skip(stride)).map(|items| items.len())
    }

    // Options and opcodes are prefixed by a u8 count instead of a u32.
    fn options(&mut self) -> Result<Vec<u64>, MessageError> {
        (0..self.u8()?).map(|_| self.u64()).collect()
    }
}
