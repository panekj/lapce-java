use std::{
    fs::{self, File},
    io::{self},
    path::PathBuf,
};

use anyhow::Result;

use flate2::read::MultiGzDecoder;
use lapce_plugin::{
    psp_types::{
        lsp_types::{request::Initialize, DocumentFilter, DocumentSelector, InitializeParams, Url},
        Request,
    },
    register_plugin, Http, LapcePlugin, VoltEnvironment, PLUGIN_RPC,
};
use serde_json::Value;

#[derive(Default)]
struct State {}

register_plugin!(State);

fn initialize(params: InitializeParams) -> Result<()> {
    let document_selector: DocumentSelector = vec![DocumentFilter {
        // lsp language id
        language: Some(String::from("java")),
        // glob pattern
        pattern: Some(String::from("**/*.java")),
        // like file:
        scheme: None,
    }];
    let mut server_args = vec![];
    if let Some(options) = params.initialization_options.as_ref() {
        if let Some(lsp) = options.get("lsp") {
            if let Some(args) = lsp.get("serverArgs") {
                if let Some(args) = args.as_array() {
                    if !args.is_empty() {
                        server_args = vec![];
                    }
                    for arg in args {
                        if let Some(arg) = arg.as_str() {
                            server_args.push(arg.to_string());
                        }
                    }
                }
            }

            if let Some(server_path) = lsp.get("serverPath") {
                if let Some(server_path) = server_path.as_str() {
                    if !server_path.is_empty() {
                        let server_uri = Url::parse(&format!("urn:{}", server_path))?;
                        PLUGIN_RPC.start_lsp(
                            server_uri,
                            server_args,
                            document_selector,
                            params.initialization_options,
                        );
                        return Ok(());
                    }
                }
            }
        }
    }

    let file_name = "jdt-language-server-latest";
    let dir = PathBuf::from(file_name);
    let gz_path = PathBuf::from(format!("{file_name}.tar.gz"));
    let url = format!(
        "http://download.eclipse.org/jdtls/snapshots/{}.tar.gz",
        file_name
    );

    if !PathBuf::from(file_name).exists() {
        if !gz_path.exists() {
            let mut resp = Http::get(&url)?;
            let body = resp.body_read_all()?;
            fs::write(&gz_path, body)?;
        }

        let tar_gz = fs::File::open(gz_path).unwrap();
        let tar = MultiGzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        fs::create_dir(&dir)?;
        for (_, file) in archive.entries().unwrap().raw(true).enumerate() {
            let mut entry = file?;
            let entry_type = entry.header().entry_type();
            if !entry_type.is_dir() && !entry_type.is_file() {
                continue;
            }
            let entry_path = dir.join(&entry.path()?);
            if entry_type.is_dir() {
                fs::create_dir_all(&entry_path)?;
            } else if entry_type.is_file() {
                let mut outfile = File::create(&entry_path)?;
                io::copy(&mut entry, &mut outfile)?;
            }
        }
    }

    // Plugin working directory
    let volt_uri = VoltEnvironment::uri()?;

    let base_path = Url::parse(&volt_uri)?;

    let server_uri = base_path.join(file_name)?.join("bin")?.join("jdtls")?;

    PLUGIN_RPC.stderr(&format!("Starting java lsp server: {server_uri:?}"));

    PLUGIN_RPC.start_lsp(
        server_uri,
        server_args,
        document_selector,
        params.initialization_options,
    );

    Ok(())
}

impl LapcePlugin for State {
    fn handle_request(&mut self, _id: u64, method: String, params: Value) {
        PLUGIN_RPC.stderr(&format!("{_id}, {method}"));
        #[allow(clippy::single_match)]
        match method.as_str() {
            Initialize::METHOD => {
                let params: InitializeParams = serde_json::from_value(params).unwrap();
                match initialize(params) {
                    Ok(_) => (),
                    Err(e) => {
                        PLUGIN_RPC.stderr(&format!("plugin returned with error: {e}"))
                    }
                };
            }
            _ => {}
        }
    }
}
