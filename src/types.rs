use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AxisCommand {
    pub axis: String,
    pub value: f32,
}

#[derive(Debug, Deserialize)]
pub struct ButtonCommand {
    pub button: u8,
    pub pressed: bool,
}

#[derive(Debug, Deserialize)]
pub struct AppCommand {
    pub axis: Option<AxisCommand>,
    pub button: Option<ButtonCommand>,
}

pub enum Axis {
    X = 1,
    Y = 2,
    Z = 3,
    Rx = 4,
    Ry = 5,
    Rz = 6,
    Slider = 7,
    DialSlider = 8,
    // not implemented
    Wheel = 9,
    Accel = 10,
    Brake = 11,
    Clutch = 12,
    Steering = 13,
    Aileron = 14,
    Rudder = 15,
    Throttle = 16
}

