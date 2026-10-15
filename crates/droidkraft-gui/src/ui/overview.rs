//! Per-device statistics dashboard (workspace Overview tab).

use gpui::{div, prelude::*, rgb, AnyElement};

use droidkraft_core::{BatteryStatus, PlugType};

use crate::fmt::{battery_color, fmt_mib, fmt_uptime, nonempty, temp_color, usage_color};
use crate::layout::Layout;
use crate::state::DeviceState;
use crate::theme;
use crate::ui::widgets::{bar, chart, gauge, kv, pill, stat_card};

fn opt<T: std::fmt::Display>(v: Option<T>, suffix: &str) -> String {
    v.map(|v| format!("{v}{suffix}"))
        .unwrap_or_else(|| "—".into())
}

pub fn overview(d: &DeviceState, lay: &Layout) -> AnyElement {
    let Some(det) = d.details.as_ref().filter(|x| x.online) else {
        let msg = if d.connected {
            "Collecting device details…"
        } else {
            "Device disconnected — showing no live data."
        };
        return div()
            .p_4()
            .text_sm()
            .text_color(rgb(theme::TEXT_DIM))
            .child(msg)
            .into_any_element();
    };
    let (b, m, st, c, disp, n, t, s) = (
        &det.battery,
        &det.memory,
        &det.storage,
        &det.cpu,
        &det.display,
        &det.network,
        &det.thermal,
        &det.system,
    );
    let h = &d.history;
    let charts = lay.show_charts();

    let top = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap_3()
        .child(pill(
            "Battery",
            format!("{}%", b.level),
            battery_color(b.level),
        ))
        .child(pill(
            "RAM",
            format!("{:.0}%", m.used_pct()),
            usage_color(m.used_pct()),
        ))
        .child(pill(
            "CPU",
            c.usage_pct
                .map(|u| format!("{u:.0}%"))
                .unwrap_or_else(|| format!("load {:.2}", c.load_1)),
            theme::TEXT,
        ))
        .child(pill(
            "Storage",
            format!("{:.0}%", st.used_pct()),
            usage_color(st.used_pct()),
        ))
        .child(pill("Uptime", fmt_uptime(s.uptime_secs), theme::TEXT));

    let battery = stat_card("Battery")
        .child(gauge(
            "Level",
            format!("{}%", b.level),
            b.level as f32,
            battery_color(b.level),
        ))
        .child(kv("Status", format!("{:?}", b.status)))
        .child(kv("Plug", format!("{:?}", b.plugged)))
        .child(kv(
            "Temperature",
            opt(b.temperature_c.map(|t| format!("{t:.1}")), " °C"),
        ))
        .child(kv("Voltage", opt(b.voltage_mv, " mV")))
        .child(kv("Health", nonempty(&b.health, "—")))
        .child(kv("Cycles", opt(b.cycle_count, "")))
        .children(charts.then(|| {
            chart(
                "History",
                format!("{}%", b.level),
                &h.battery.as_vec_f64(),
                100.0,
                battery_color(b.level),
                36.0,
            )
        }));
    let charging = matches!(b.status, BatteryStatus::Charging)
        || !matches!(b.plugged, PlugType::None | PlugType::Unknown);
    let battery = if charging {
        battery.child(
            div()
                .text_xs()
                .text_color(rgb(theme::OK))
                .child("⚡ charging / plugged"),
        )
    } else {
        battery
    };

    let memory = stat_card("Memory")
        .child(gauge(
            "Used",
            format!("{} / {}", fmt_mib(m.used_mib()), fmt_mib(m.total_mib)),
            m.used_pct(),
            usage_color(m.used_pct()),
        ))
        .child(kv("Cached", fmt_mib(m.cached_mib)))
        .child(kv(
            "Swap",
            format!(
                "{} / {}",
                fmt_mib(m.swap_total_mib.saturating_sub(m.swap_free_mib)),
                fmt_mib(m.swap_total_mib)
            ),
        ))
        .children(charts.then(|| {
            chart(
                "History",
                format!("{:.0}%", m.used_pct()),
                &h.ram_pct.as_vec_f64(),
                100.0,
                usage_color(m.used_pct()),
                36.0,
            )
        }));

    let cpu_now = h.cpu_pct.last().copied().unwrap_or(0.0) as f32;
    let cpu = stat_card("CPU")
        .child(gauge(
            "Usage",
            format!("{cpu_now:.0}%"),
            cpu_now,
            usage_color(cpu_now),
        ))
        .child(kv("Cores", c.cores.to_string()))
        .child(kv(
            "Load",
            format!("{:.2} / {:.2} / {:.2}", c.load_1, c.load_5, c.load_15),
        ))
        .child(kv(
            "Frequency",
            format!(
                "{} / {} MHz",
                opt(c.cur_freq_mhz, ""),
                opt(c.max_freq_mhz, "")
            ),
        ))
        .child(kv("SoC", nonempty(&c.hardware, "—")))
        .child(kv("ABI", nonempty(&c.abi, "—")))
        .children(charts.then(|| {
            chart(
                "History",
                format!("{cpu_now:.0}%"),
                &h.cpu_pct.as_vec_f64(),
                100.0,
                usage_color(cpu_now),
                36.0,
            )
        }));

    let storage = stat_card("Storage /data")
        .child(gauge(
            "Used",
            format!("{} / {}", fmt_mib(st.used_mib), fmt_mib(st.total_mib)),
            st.used_pct(),
            usage_color(st.used_pct()),
        ))
        .child(kv("Free", fmt_mib(st.avail_mib)));

    let max_temp = t.cpu_temp_c.or(b.temperature_c).unwrap_or(0.0);
    let thermal = stat_card("Thermal")
        .child(kv(
            "CPU",
            opt(t.cpu_temp_c.map(|v| format!("{v:.1}")), " °C"),
        ))
        .child(kv(
            "Skin",
            opt(t.skin_temp_c.map(|v| format!("{v:.1}")), " °C"),
        ))
        .child(kv(
            "Battery",
            opt(t.battery_temp_c.map(|v| format!("{v:.1}")), " °C"),
        ))
        .child(kv("Throttling", if t.throttling { "yes" } else { "no" }))
        .child(bar(max_temp / 80.0 * 100.0, temp_color(max_temp), 6.0))
        .children(charts.then(|| {
            chart(
                "History",
                format!("{max_temp:.1} °C"),
                &h.temp.as_vec_f64(),
                80.0,
                temp_color(max_temp),
                28.0,
            )
        }));

    let display = stat_card("Display")
        .child(kv("Resolution", format!("{}×{}", disp.width, disp.height)))
        .child(kv("Density", format!("{} dpi", disp.density)))
        .child(kv(
            "Refresh",
            opt(disp.refresh_hz.map(|v| format!("{v:.0}")), " Hz"),
        ))
        .child(kv("Brightness", opt(disp.brightness_pct, "%")));

    let network = stat_card("Network")
        .child(kv(
            "SSID",
            n.wifi_ssid.clone().unwrap_or_else(|| "—".into()),
        ))
        .child(kv("IP", n.wifi_ip.clone().unwrap_or_else(|| "—".into())))
        .child(kv("RSSI", opt(n.rssi_dbm, " dBm")))
        .child(kv("Link speed", opt(n.link_mbps, " Mbps")))
        .child(kv(
            "Operator",
            n.mobile_operator.clone().unwrap_or_else(|| "—".into()),
        ))
        .child(kv("Airplane", if n.airplane { "on" } else { "off" }));

    let system = stat_card("System")
        .child(kv("Manufacturer", nonempty(&s.manufacturer, "—")))
        .child(kv("Brand", nonempty(&s.brand, "—")))
        .child(kv("Model", nonempty(&s.model, "—")))
        .child(kv("Codename", nonempty(&s.codename, "—")))
        .child(kv(
            "Android",
            format!("{} (SDK {})", s.android_release, s.sdk),
        ))
        .child(kv("Security patch", nonempty(&s.security_patch, "—")))
        .child(kv("Build ID", nonempty(&s.build_id, "—")))
        .child(kv("Fingerprint", nonempty(&s.build_fingerprint, "—")))
        .child(kv("Kernel", nonempty(&s.kernel, "—")))
        .child(kv("Bootloader", nonempty(&s.bootloader, "—")))
        .child(kv("Baseband", nonempty(&s.baseband, "—")))
        .child(kv("Verified boot", nonempty(&s.verified_boot_state, "—")))
        .child(kv("Treble", if s.treble { "yes" } else { "no" }))
        .child(kv("Timezone", nonempty(&s.timezone, "—")))
        .child(kv("Locale", nonempty(&s.locale, "—")));

    let ncols = lay.overview_columns();
    let col = |c: gpui::Div| c.flex().flex_col().gap_3().flex_1().min_w_0();
    let panels: Vec<AnyElement> = vec![
        battery.into_any_element(),
        display.into_any_element(),
        memory.into_any_element(),
        storage.into_any_element(),
        thermal.into_any_element(),
        cpu.into_any_element(),
        network.into_any_element(),
    ];
    let mut cols: Vec<gpui::Div> = (0..ncols).map(|_| col(div())).collect();
    for (i, p) in panels.into_iter().enumerate() {
        let k = i % ncols;
        let c = std::mem::replace(&mut cols[k], div());
        cols[k] = c.child(p);
    }
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(top)
        .child(div().flex().flex_row().gap_3().items_start().children(cols))
        .child(system)
        .into_any_element()
}
