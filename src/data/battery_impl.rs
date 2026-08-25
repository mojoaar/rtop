use crate::data::snapshot::BatteryInfo;

pub fn read_battery() -> Option<BatteryInfo> {
    let manager = battery::Manager::new().ok()?;
    let mut total_capacity = 0.0f32;
    let mut sum_capacity = 0.0f32;
    let mut state = String::new();
    let mut time_to_full: Option<u64> = None;
    let mut time_to_empty: Option<u64> = None;
    let mut any = false;

    for b in manager.batteries().ok()? {
        let Ok(battery) = b else { continue };
        any = true;
        let cap = battery.energy_full().value as f32;
        let now = battery.energy().value as f32;
        total_capacity += cap;
        sum_capacity += now;
        state = format!("{:?}", battery.state());
        time_to_full = battery.time_to_full().map(|t| t.value as u64);
        time_to_empty = battery.time_to_empty().map(|t| t.value as u64);
    }

    if !any || total_capacity <= 0.0 {
        return None;
    }

    Some(BatteryInfo {
        percentage: (sum_capacity / total_capacity * 100.0).clamp(0.0, 100.0),
        state,
        time_to_full_secs: time_to_full,
        time_to_empty_secs: time_to_empty,
    })
}
