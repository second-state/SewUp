use std::{fs::File, path::Path};

use anyhow::{anyhow, Context, Result};
use fancy_regex::Regex;
use hex::encode;
use sha2::{Digest, Sha256};
use tokio::{
    fs::{read, read_to_string, write},
    process::Command,
};
use wasmprinter::print_file;

use cargo_sewup::config::{CargoLock, CargoToml, Package};
use cargo_sewup::deploy_wasm;

async fn check_cargo_toml() -> Result<String> {
    let config_contents = read_to_string("Cargo.toml")
        .await
        .context("can not read Cargo.toml")?;
    let config: CargoToml = toml::from_str(config_contents.as_str())?;

    let mut has_feautes_error = false;

    if let Some(features) = config.features {
        if features.constructor.is_none() || features.constructor_test.is_none() {
            has_feautes_error = true;
        }
    } else {
        has_feautes_error = true;
    }

    if has_feautes_error {
        return Err(anyhow!(
            "There should be features following: constructor, constructor-test"
        ));
    }
    let mut has_profile_error = false;
    if let Some(profile) = config.profile {
        if let Some(release) = profile.release {
            if release.incremental != Some(false)
                || release.panic != Some("abort".to_string())
                || release.lto != Some(true)
                || release.opt_level != Some("z".to_string())
            {
                has_profile_error = true;
            }
        } else {
            has_profile_error = true;
        }
    } else {
        has_profile_error = true;
    }
    if has_profile_error {
        return Err(anyhow!(
            r#"Please provide [profile.release] section as following:
[profile.release]
incremental = false
panic = "abort"
lto = true
opt-level = "z""#
        ));
    }

    Ok(config.package.name.replace("-", "_"))
}

async fn sewup_version_check() -> Result<()> {
    Command::new("cargo")
        .args(&["generate-lockfile"])
        .output()
        .await
        .context("fail to generate lock")?;

    let lock_contents = read_to_string("Cargo.lock")
        .await
        .context("can not read Cargo.lock")?;
    let package_locks: CargoLock = toml::from_str(lock_contents.as_str())?;
    let mut sewup_version = None;
    let mut sewup_derive_version = None;
    for Package { name, version } in package_locks.package.into_iter() {
        if name == "sewup" {
            sewup_version = Some(version)
        } else if name == "sewup-derive" {
            sewup_derive_version = Some(version)
        }
    }
    let mut warning_msg =
        "Warning: following sewup crates are not in the same version\nCargo Sewup:".to_string();
    let mut warning = false;
    let cargo_sewup_version = env!("CARGO_PKG_VERSION");
    warning_msg.push_str(cargo_sewup_version);
    warning_msg.push('\n');

    if let Some(sewup_version) = sewup_version {
        warning_msg.push_str("Sewup: ");
        warning_msg.push_str(&sewup_version);
        warning_msg.push('\n');
        if sewup_version != cargo_sewup_version {
            warning = true;
        }
        if let Some(sewup_derive_version) = sewup_derive_version {
            warning_msg.push_str("Sewup Derive: ");
            warning_msg.push_str(&sewup_derive_version);
            warning_msg.push('\n');
            if sewup_derive_version != cargo_sewup_version || sewup_derive_version != sewup_version
            {
                warning = true;
            }
        }
    }

    if warning {
        println!("{}", warning_msg);
    }

    Ok(())
}

async fn build_constructor_template(contract_wasm_path: &str) -> Result<String> {
    Command::new("cargo")
        .args(&[
            "build",
            "--release",
            "--target=wasm32-unknown-unknown",
            "--features=constructor",
        ])
        .output()
        .await
        .context("fail to build runtime wasm")?;
    let wit = print_file(contract_wasm_path)?;
    Ok(wit)
}

async fn build_runtime_wat(contract_wasm_path: &str) -> Result<String> {
    Command::new("cargo")
        .args(&["build", "--release", "--target=wasm32-unknown-unknown"])
        .output()
        .await
        .context("fail to build runtime wasm")?;
    let rt_content = print_file(contract_wasm_path)?;

    let hinden_export_re = unsafe { Regex::new(r#"\(export "__.*\)\)\n"#).unwrap_unchecked() };
    Ok(hinden_export_re
        .replace_all(&rt_content, "")
        .trim_end()
        .to_string())
}

async fn generate_runtime_info(wat_content: String) -> Result<(usize, usize, String)> {
    let bin = wat::parse_str(wat_content).context("fail to rebuild runtime wasm")?;
    let bin_size = bin.len();
    let mem_size = bin_size / (64 * 1024) + 1;
    let hex_string = encode(bin)
        .chars()
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|x| format!(r#"\{}{}"#, x[0], x[1]))
        .fold(String::new(), |mut acc, x| {
            acc.push_str(&x);
            acc
        });
    Ok((bin_size, mem_size, hex_string))
}

async fn build_wat(
    tmpl: String,
    bin_size: usize,
    mem_size: usize,
    hex_string: String,
) -> Result<String> {
    let eth_finish_re = unsafe {
        Regex::new(r#"\(import "ethereum" "finish" \(func (?P<eth_finish_sig>[^\s]*) "#)
            .unwrap_unchecked()
    };

    let eth_finish_sig = if let Ok(Some(captures)) = eth_finish_re.captures(&tmpl) {
        captures.name("eth_finish_sig").map(|c| c.as_str())
    } else {
        None
    };

    let memory_re =
        unsafe { Regex::new(r#"\(memory \(;0;\) (?P<mem_size>\d*)"#).unwrap_unchecked() };
    let mem_size = if let Ok(Some(cap)) = memory_re.captures(&tmpl) {
        cap.name("mem_size")
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap()
            + mem_size
    } else {
        mem_size
    };

    let mut content = memory_re
        .replace(&tmpl, &format!(r#"(memory (;0;) {}"#, mem_size))
        .trim_end()
        .to_string();

    content.truncate(content.len() - 1);

    let export_re = unsafe { Regex::new(r#"\(export.*\)\)\n"#).unwrap_unchecked() };
    content = export_re.replace_all(&content, "").trim_end().to_string();

    let main_call = if let Some(eth_finish_sig) = eth_finish_sig {
        format!(
            r#"
  (export "memory" (memory 0))
  (func (export "main") call $__constructor i32.const 0 i32.const {} call {})"#,
            bin_size, eth_finish_sig
        )
    } else {
        let module_re = unsafe { Regex::new(r#"\n\s*\(func \$"#).unwrap_unchecked() };
        content = module_re
            .replace(
                &content,
                r#"
  (import "ethereum" "finish" (func $$_Eth_Finish (param i32 i32)))
  (func $$"#,
            )
            .trim_end()
            .to_string();
        format!(
            r#"
  (export "memory" (memory 0))
  (func (export "main") call $__constructor i32.const 0 i32.const {} call $_Eth_Finish)
        "#,
            bin_size
        )
    };
    content.push_str(&main_call);
    let data_section = format!(
        r#"
  (data (i32.const 0) "{}")"#,
        hex_string
    );
    content.push_str(&data_section);
    content.push(')');

    Ok(content)
}

async fn build_deploy_wasm(wat_content: String, wasm_path: &str) -> Result<()> {
    let binary = wat::parse_str(wat_content)?;
    write(wasm_path, binary).await?;
    Ok(())
}

async fn generate_deploy_wasm_hex(wasm_path: &str, text_path: &str) -> Result<()> {
    let bin = read(Path::new(wasm_path)).await?;
    let hex_string = encode(bin);
    write(text_path, hex_string)
        .await
        .context("fail to generate hex string for deploy wasm")?;
    Ok(())
}

async fn generate_debug_wat(wat_path: &str, wat_content: &str) -> Result<()> {
    write(wat_path, wat_content)
        .await
        .context("fail to write deploy wat")?;
    Ok(())
}

async fn list_fn_sig() -> Result<Vec<(String, String)>> {
    let builder = tempfile::Builder::new();
    let outdir = builder.tempdir().expect("failed to create tmp file");
    let outfile_path = outdir.path().join("expanded");

    Command::new("cargo")
        .args(&[
            "rustc",
            "--target=wasm32-unknown-unknown",
            "--",
            "-o",
            outfile_path.to_str().unwrap(),
            "-Zunpretty=expanded",
        ])
        .output()
        .await
        .context("fail to expand macro")?;
    let expanded = read_to_string(outfile_path).await?;

    let sig_re = unsafe {
        Regex::new(r"(?P<sig_name>[A-Za-z0-9:_]*)_SIG: \[u8; 4\] = \[(?P<sig_value>[0-9u,\s]*)\];")
            .unwrap_unchecked()
    };
    Ok(sig_re
        .captures_iter(&expanded)
        .filter(|c| c.is_ok())
        .map(|c| {
            let sig_name = c.as_ref().unwrap().name("sig_name").unwrap().as_str();
            let sig_values: Vec<String> = c
                .unwrap()
                .name("sig_value")
                .unwrap()
                .as_str()
                .replace("u8", "")
                .split(',')
                .map(|p| p.trim().into())
                .collect();
            let sig_hex_str = sig_values
                .iter()
                .map(|b| format!("{:02x}", b.parse::<u8>().unwrap()))
                .collect::<Vec<_>>()
                .join("");

            (sig_name.into(), sig_hex_str)
        })
        .collect())
}

async fn build(debug: bool, contract_name: &str) -> Result<String> {
    let mut wasm_path = format!(
        "./target/wasm32-unknown-unknown/release/{}.wasm",
        contract_name
    );

    let wasm_tmpl = build_constructor_template(&wasm_path).await?;

    if debug {
        let tmpl_path = format!(
            "./target/wasm32-unknown-unknown/release/{}.tmpl.wat",
            contract_name
        );
        generate_debug_wat(&tmpl_path, &wasm_tmpl).await?;
    }

    let rt_content = build_runtime_wat(&wasm_path).await?;

    if debug {
        let rt_path = format!(
            "./target/wasm32-unknown-unknown/release/{}.rt.wat",
            contract_name
        );
        generate_debug_wat(&rt_path, &rt_content).await?;
    }

    let (bin_size, mem_size, hex_string) = generate_runtime_info(rt_content).await?;

    let wat_content = build_wat(wasm_tmpl, bin_size, mem_size, hex_string).await?;

    if debug {
        let wat_path = format!(
            "./target/wasm32-unknown-unknown/release/{}.wat",
            contract_name
        );
        generate_debug_wat(&wat_path, &wat_content).await?;
    }

    wasm_path = format!(deploy_wasm!(), contract_name);
    build_deploy_wasm(wat_content, &wasm_path).await?;

    if debug {
        let text_path = format!(
            "./target/wasm32-unknown-unknown/release/{}.deploy",
            contract_name
        );
        generate_deploy_wasm_hex(&wasm_path, &text_path).await?;
    }
    let mut file = File::open(wasm_path)?;
    let mut sha256 = Sha256::new();
    std::io::copy(&mut file, &mut sha256)?;
    let hash: [u8; 32] = sha256
        .finalize()
        .as_slice()
        .try_into()
        .expect("hash size unexpected");
    let hex_str = hash
        .iter()
        .map(|b| format!("{:02x}", *b))
        .collect::<Vec<_>>()
        .join("");
    Ok(hex_str)
}

async fn get_version() -> Result<String> {
    let version_info = Command::new("rustc")
        .args(&["--version"])
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("fail to get rustc version: {:?}", e))?;
    Ok(std::str::from_utf8(&version_info.stdout)
        .expect("output of rustc version should be utf-8 decoded")
        .trim()
        .into())
}

pub async fn run(debug: bool) -> Result<String> {
    let contract_name = check_cargo_toml().await?;

    match tokio::try_join!(
        build(debug, &contract_name),
        list_fn_sig(),
        get_version(),
        sewup_version_check()
    ) {
        Ok((hex_str, fn_sigs, version, _)) => {
            let meta_path = format!(
                "./target/wasm32-unknown-unknown/release/{}.metadata.toml",
                contract_name
            );
            let mut meta_content = format!(
                r#"[metadata]
name = "{}"
deploy_wasm_sha256 = "{}"
rustc = "{}"
cargo-sewup = "{}"

[function]
"#,
                &contract_name,
                hex_str,
                version,
                env!("CARGO_PKG_VERSION")
            );
            for (fn_name, fn_sig) in fn_sigs {
                meta_content = meta_content + &fn_name;
                meta_content += r#" = ""#;
                meta_content = meta_content + &fn_sig;
                meta_content += r#"""#;
                meta_content += "\n";
            }
            write(meta_path, meta_content).await?;
        }
        Err(err) => return Err(err),
    };

    Ok(contract_name)
}
