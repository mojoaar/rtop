use crate::data::snapshot::FanInfo;
use crate::platform::FanStats;
use smc_lib::io::IOService;
use smc_lib::value::SmcValue;

pub struct MacFan;

// smc-lib reads SMC keys over IOKit. Fan keys: "FNum" (count) and
// "F0Ac", "F1Ac", ... (actual RPM, a 4-byte `flt` value).
//
// On Apple Silicon or on systems without an accessible SMC fan driver, opening
// the connection or reading "FNum" fails, and we return an empty Vec gracefully.
impl FanStats for MacFan {
    fn read(&self) -> Vec<FanInfo> {
        let mut fans = Vec::new();

        let smc = match IOService::init() {
            Ok(service) => service,
            Err(_) => return fans,
        };

        let count = match smc.read_key(b"FNum").ok().and_then(|v| v.data_value()) {
            Some(SmcValue::U8(n)) => n as usize,
            _ => return fans,
        };

        for i in 0..count {
            // SMC keys are exactly 4 ASCII bytes; we only support fan indices 0..=9.
            if i > 9 {
                break;
            }
            let key = [b'F', b'0' + i as u8, b'A', b'c'];
            let rpm = match smc.read_key(&key).ok().and_then(|v| v.data_value()) {
                Some(SmcValue::F32 { le, .. }) if le.is_finite() && le > 0.0 => le as u32,
                Some(SmcValue::U16(v)) => v as u32,
                Some(SmcValue::U32(v)) => v,
                _ => continue,
            };
            fans.push(FanInfo {
                label: format!("Fan {}", i),
                rpm,
            });
        }

        fans
    }
}
