use std::{env, fs, path::PathBuf};

use anyhow::anyhow;

fn main() -> anyhow::Result<()> {
    extract_custom_domain()?;

    Ok(())
}

fn extract_custom_domain() -> anyhow::Result<()> {
    let wrangler_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wrangler.toml");

    println!("cargo:rerun-if-changed={}", wrangler_path.display());

    let wrangler = fs::read_to_string(&wrangler_path)?;
    let wrangler: toml::Value = toml::from_str(&wrangler)?;
    let custom_domain = wrangler
        .get("routes")
        .ok_or(anyhow!("no `routes` found in `wrangler.toml`"))?
        .as_array()
        .ok_or(anyhow!("`routes` found in `wrangler.toml` is not an array"))?
        .iter()
        .find_map(|route| {
            let route = route.as_table()?;
            let custom_domain = route.get("custom_domain")?.as_bool()?;
            let pattern = route.get("pattern")?.as_str()?;
            custom_domain.then(|| pattern.to_string())
        })
        .ok_or(anyhow!("no custom domain is specified in `wrangler.toml`"))?;

    let callback_url = format!("https://{custom_domain}/callback");
    println!("cargo:rustc-env=OAUTH_CALLBACK_URL={callback_url}");

    Ok(())
}
