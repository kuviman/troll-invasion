use ::*;

#[derive(Copy, Clone)]
pub enum PlayType {
    Player,
    Spectator,
}

impl std::str::FromStr for PlayType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "player" => Ok(PlayType::Player),
            "spectator" => Ok(PlayType::Spectator),
            _ => Err(()),
        }
    }
}

pub enum ServerMessage {
    ReadyStatus {
        nick: String,
        ready: bool,
    },
    MapLine(usize, Vec<Option<GameCell>>),
    GameStart,
    PlayerColor {
        nick: String,
        color: char,
    },
    Turn {
        nick: String,
    },
    SelectCell {
        row: usize,
        col: usize,
    },
    DeselectCell,
    GameFinish {
        winner: String,
    },
    UpgradePhase,
    EnergyLeft(usize),
    GameList {
        name: String,
        player_count: usize,
    },
    GameLeft {
        nick: String
    },
    GameEntered {
        name: String,
        typ: PlayType,
    },
    HoverCell {
        nick: String,
        row: usize,
        col: usize,
    },
    HoverNone {
        nick: String,
    },
    SpectatorJoin {
        nick: String,
    },
    CanMove {
        cells: Vec<Vec2<usize>>,
    },
}

impl ServerMessage {
    pub fn parse(message: &str) -> Option<Self> {
        use ServerMessage::*;
        let mut args = message.split_whitespace();
        let command = args.next().unwrap();
        Some(match command {
            "readyStatus" => ReadyStatus {
                nick: args.next().unwrap().to_owned(),
                ready: args.next().unwrap().parse().unwrap(),
            },
            "gameStart" => GameStart,
            "playerColor" => PlayerColor {
                nick: args.next().unwrap().to_owned(),
                color: args.next().unwrap().parse().unwrap(),
            },
            "turn" => Turn {
                nick: args.next().unwrap().to_owned(),
            },
            "selectCell" => SelectCell {
                row: args.next().unwrap().parse().unwrap(),
                col: args.next().unwrap().parse().unwrap(),
            },
            "deselectCell" => DeselectCell,
            "gameFinish" => GameFinish {
                winner: args.next().unwrap().to_owned(),
            },
            "upgradePhase" => UpgradePhase,
            "energyLeft" => EnergyLeft(args.next().unwrap().parse().unwrap()),
            "mapLine" => {
                let index = args.next().unwrap().parse().unwrap();
                let cells = args.next().unwrap().split('|').map(|cell| {
                    match cell {
                        "##" => Some(GameCell::Empty),
                        "__" => None,
                        _ => {
                            let (count, owner) = cell.split_at(cell.len() - 1);
                            let count = count.parse().unwrap();
                            let owner = owner.parse().unwrap();
                            Some(GameCell::Populated {
                                count,
                                owner,
                            })
                        }
                    }
                }).collect();
                MapLine(index, cells)
            }
            "gameList" => GameList {
                name: args.next().unwrap().to_owned(),
                player_count: args.next().unwrap().parse().unwrap(),
            },
            "gameEntered" => GameEntered {
                name: args.next().unwrap().to_owned(),
                typ: args.next().unwrap().parse().unwrap(),
            },
            "gameLeft" => GameLeft {
                nick: args.next().unwrap().to_owned(),
            },
            "hover" => {
                let nick = args.next().unwrap().to_owned();
                let next = args.next().unwrap();
                if next == "none" {
                    HoverNone { nick }
                } else {
                    HoverCell {
                        nick,
                        row: next.parse().unwrap(),
                        col: args.next().unwrap().parse().unwrap(),
                    }
                }
            }
            "spectatorJoin" => SpectatorJoin { nick: args.next().unwrap().parse().unwrap() },
            "canMove" => CanMove {
                cells: {
                    let mut cells = Vec::new();
                    while let Some(arg) = args.next() {
                        let row: usize = arg.parse().unwrap();
                        let col: usize = args.next().unwrap().parse().unwrap();
                        cells.push(vec2(row, col));
                    }
                    cells
                }
            },
            _ => return None
        })
    }
}

#[derive(Copy, Clone)]
pub enum GameCell {
    Empty,
    Populated {
        count: usize,
        owner: char,
    }
}