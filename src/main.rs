use eframe::{run_native, NativeOptions, Renderer, egui_wgpu::WgpuConfiguration, wgpu::PowerPreference};
use mandelsnot::Mandelsnot;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let options = NativeOptions {
        default_theme: eframe::Theme::Dark,
        follow_system_theme: false,
        renderer: Renderer::Wgpu,
        wgpu_options: WgpuConfiguration {
            power_preference: PowerPreference::LowPower,
            ..Default::default()
        },
        ..Default::default()
    };

    run_native(
        Mandelsnot::APP_NAME,
        options,
        Box::new(|cc| Mandelsnot::new(cc)),
    )?;

    Ok(())
}
