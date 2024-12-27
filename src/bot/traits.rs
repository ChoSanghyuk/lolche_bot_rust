use std::fmt::format;


pub trait DeckLoader<E: std::error::Error> {
    fn load_deck(&self, mode:Mode) -> Result<Vec<String>, E>;
    fn fix_css() -> Result<(), E>;
}

pub trait Storage <E>{

}

#[derive(Clone)]
pub enum Mode{
    main,
    pbe
}

impl Mode {
    pub fn switch(self) -> Self{
        match self {
            Mode::main => return Mode::pbe,
            Mode::pbe => return Mode::main,
        }
    }

    pub fn msg(&self) -> String {
        match self {
            Mode::main => return format!("정규 모드"),
            Mode::pbe => return format!("pbe 모드"),
        }
    }
}