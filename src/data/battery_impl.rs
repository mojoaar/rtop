use crate::data::snapshot::BatteryInfo;

pub fn read_battery() -> Option<BatteryInfo> {
    let manager = battery::Manager::new().ok()?;
    let mut total_capacity = 0.0f32;
    let mut sum_capacity = 0.0f32;
    let mut primary: Option<battery::Battery> = None;
    let mut any = false;

    for b in manager.batteries().ok()? {
        let Ok(battery) = b else { continue };
        any = true;
        let cap = battery.energy_full().value;
        let now = battery.energy().value;
        total_capacity += cap;
        sum_capacity += now;
        if primary.is_none() {
            primary = Some(battery);
        }
    }

    if !any || total_capacity <= 0.0 {
        return None;
    }

    let state = match primary.as_ref().map(|b| b.state()) {
        Some(battery::State::Charging) => "charging",
        Some(battery::State::Discharging) => "discharging",
        Some(battery::State::Full) => "full",
        Some(battery::State::Empty) => "empty",
        _ => "on AC",
    }
    .to_string();

    Some(BatteryInfo {
        percentage: (sum_capacity / total_capacity * 100.0).clamp(0.0, 100.0),
        state,
        time_to_full_secs: primary
            .as_ref()
            .and_then(|b| b.time_to_full())
            .map(|t| t.value as u64),
        time_to_empty_secs: primary
            .as_ref()
            .and_then(|b| b.time_to_empty())
            .map(|t| t.value as u64),
        cycle_count: primary.as_ref().and_then(|b| b.cycle_count()),
        health_percent: primary
            .as_ref()
            .map(|b| (b.state_of_health().value * 100.0).clamp(0.0, 100.0)),
    })
}
