use std::path::PathBuf;

use crate::types::Axis;

pub fn map_axis(name: &str) -> Option<Axis> {
    match name.to_lowercase().as_str() {
        "x" => Some(Axis::X),
        "y" => Some(Axis::Y),
        "z" => Some(Axis::Z),
        "rx" => Some(Axis::Rx),
        "ry" => Some(Axis::Ry),
        "rz" => Some(Axis::Rz),
        "slider" => Some(Axis::Slider),
        "dialslider" => Some(Axis::DialSlider),
        // not implemented
        "wheel" => Some(Axis::Wheel),
        "accel" => Some(Axis::Accel),
        "brake" => Some(Axis::Brake),
        "clutch" => Some(Axis::Clutch),
        "steering" => Some(Axis::Steering),
        "aileron" => Some(Axis::Aileron),
        "rudder" => Some(Axis::Rudder),
        "throttle" => Some(Axis::Throttle),
        _ => None,
    }
}

pub fn range_to_i32(val: f32) -> i32 {
    let max = i16::MAX;

    if val <= 0.0 {
        return 0;
    }

    if val >= 1.0 {
        return max as i32;
    }
    
    return (max as f32 * val) as i32;
}

pub fn get_device_ips() -> Vec<String> {
    let ifaces = local_ip_address::list_afinet_netifas().unwrap();

    // Список с уникальными ip адресами
    let mut unique_ips = vec![];

    for (_name, ip) in ifaces {
        if unique_ips.contains(&ip.to_string())
            || !ip.is_ipv4()
            || ip.is_loopback()
            || ip.is_multicast()
            || ip.is_unspecified()
        {
            continue;
        }

        unique_ips.push(ip.to_string());

        // println!("{}:\t{:?}", name, ip);
    }

    unique_ips
}

pub fn get_cert_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("certs")
    .join("cert.pem")
}
pub fn get_key_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("certs")
    .join("key.pem")
}