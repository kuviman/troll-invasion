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
    GameFinish,
    UpgradePhase,
    EnergyLeft(usize),
    GameList {
        name: String,
        player_count: usize,
    }
}

impl ServerMessage {
    pub fn parse(message: &str) -> Self {
        use ServerMessage::*;
        let mut args = message.split_whitespace();
        let command = args.next().unwrap();
        match command {
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
            "gameFinish" => GameFinish,
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
            "gameList" => {
                GameList {
                    name: args.next().unwrap().to_owned(),
                    player_count: args.next().unwrap().parse().unwrap(),
                }
            }
            _ => panic!("Unexpected message: {:?}", message)
        }
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