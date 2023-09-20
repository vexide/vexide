pub struct Adi {
    ports: [AdiPort; pros_sys::NUM_ADI_PORTS as usize]
}

enum AdiType {
    InputDigital,
    InputAnalog,
    OutputDigital,
    OutputAnalog,
}

struct AdiPort {
    adi_type: AdiType,
}