use dropbear_engine::gilrs;

pub fn button_from_ordinal(ordinal: i32) -> Result<gilrs::Button, ()> {
    match ordinal {
        0 => Ok(gilrs::Button::Unknown),
        1 => Ok(gilrs::Button::South),
        2 => Ok(gilrs::Button::East),
        3 => Ok(gilrs::Button::North),
        4 => Ok(gilrs::Button::West),
        5 => Ok(gilrs::Button::C),
        6 => Ok(gilrs::Button::Z),
        7 => Ok(gilrs::Button::LeftTrigger),
        8 => Ok(gilrs::Button::RightTrigger),
        9 => Ok(gilrs::Button::LeftTrigger2),
        10 => Ok(gilrs::Button::RightTrigger2),
        11 => Ok(gilrs::Button::Select),
        12 => Ok(gilrs::Button::Start),
        13 => Ok(gilrs::Button::Mode),
        14 => Ok(gilrs::Button::LeftThumb),
        15 => Ok(gilrs::Button::RightThumb),
        16 => Ok(gilrs::Button::DPadUp),
        17 => Ok(gilrs::Button::DPadDown),
        18 => Ok(gilrs::Button::DPadLeft),
        19 => Ok(gilrs::Button::DPadRight),
        _ => Err(()),
    }
}
