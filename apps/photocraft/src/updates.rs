//! Release checks through `craft-update`. Only public release metadata is requested.

use photocraft_engine::updates_cmds::{CheckFn, CheckResult, Release};

pub const ANGKORPIX_APP: craft_update::App = craft_update::App {
    id: "angkorpix",
    name: "AngkorPix",
    tagline: "Professional Image Editor",
    repo: "itbtdeveloper123-png/AngkorPix",
    former_ids: &[],
    former_names: &[],
    icon_png: None,
};

pub fn service() -> CheckFn {
    Box::new(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("update check".into())
            .spawn(move || {
                let result = check();
                if let Err(e) = &result {
                    log::warn!("Update check: {e}");
                }
                let _ = tx.send(result);
            })
            .map_err(|e| format!("Could not start update check: {e}"))?;
        Ok(rx)
    })
}

fn check() -> CheckResult {
    let http = craft_update::Http::new("AngkorPix/Update")?;
    let found = craft_update::check_for_update(&ANGKORPIX_APP, &http, photocraft_engine::build_info::VERSION)?;
    Ok(found.map(|release| Release { version: release.tag.trim_start_matches(['v', 'V']).to_string(), url: release.html_url }))
}
