extern crate proc_macro;
use std::fs;

use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

// TODO: Share this code
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Template {
    abi: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataSource {
    abi: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    data_sources: HashMap<String, DataSource>,
    templates: HashMap<String, Template>,
}
//

#[proc_macro_attribute]
pub fn handler(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let metadata_string = metadata.to_string();
    let mut metadata_split = metadata_string.split(".");

    let name = metadata_split.next();
    let event_name = metadata_split.next();

    if name.is_none() {
        panic!("The source is missing");
    }

    if event_name.is_none() {
        panic!("The event name is missing");
    }

    // Checks that the metadata does not have more than 3 comma separated values
    let should_be_none = metadata_split.next();
    if should_be_none.is_some() {
        panic!("The metadata has too many values");
    }

    let name = name.unwrap();
    let name = String::from(name.trim());

    let event_name = event_name.unwrap();
    let event_name = String::from(event_name.trim());

    if name.len() == 0 {
        panic!("The source is empty");
    }

    if event_name.len() == 0 {
        panic!("The event name is empty");
    }

    let current_dir = std::env::current_dir().unwrap();
    let content = fs::read_to_string(current_dir.join("config.json"));

    let abi;
    let mut is_template = false;

    match content {
        Ok(content) => {
            let config: Config = serde_json::from_str(&content).unwrap();
            let source_data_source = config.data_sources.get(&name);
            let source_template = config.templates.get(&name);

            if source_data_source.is_none() && source_template.is_none() {
                panic!("Source '{}' not found.", name);
            }

            if source_data_source.is_some() {
                abi = source_data_source.unwrap().abi.clone()
            } else {
                is_template = true;
                abi = source_template.unwrap().abi.clone()
            }
        }
        Err(err) => {
            panic!("Error reading the config.json file: {}", err);
        }
    };

    let abi = Literal::string(&abi);
    let event_name = syn::Ident::new(&event_name, proc_macro2::Span::call_site());

    let parsed = parse_macro_input!(input as ItemFn);
    let fn_name = parsed.sig.ident;
    let fn_body = parsed.block;
    let fn_args = parsed.sig.inputs;
    let contract_name = format_ident!("{}Contract", fn_name);

    let data_source = Literal::string(&name);

    let data_source_init = if is_template {
        quote! {}
    } else {
        quote! {
            pub fn init() {
                let config = config::load();
                let source = config.data_sources.get(#data_source).unwrap();

                let run_input = vec![HandlerConfig {
                    start_block: source.start_block,
                    step: 10_000,
                    address: source.address.clone(),
                    handler: Arc::new(#fn_name::new()),
                    network: source.network.clone(),
                }];

                tokio::spawn(async move {
                    run(run_input).await;
                });
            }
        }
    };

    TokenStream::from(quote! {
        sol!(
            #[sol(rpc)]
            #contract_name,
            #abi
        );

        pub struct #fn_name {}

        impl #fn_name {
            pub fn new() -> Box<#fn_name> {
                Box::new(#fn_name {})
            }
        }

        impl #fn_name {
            pub fn start(address: &str, start_block: u64) {
                let config = config::load();
                let source = config.templates.get(#data_source).unwrap();

                let run_input = vec![HandlerConfig {
                    start_block: start_block,
                    step: 10_000,
                    address: String::from(address),
                    handler: Arc::new(#fn_name::new()),
                    network: source.network.clone()
                }];

                tokio::spawn(async move {
                    run(run_input).await;
                });
            }

            #data_source_init
        }

        #[async_trait]
        impl Handler for #fn_name {
            async fn handle(&self, #fn_args) {
                #fn_body
            }

            fn get_source(&self) -> String {
                String::from(#data_source)
            }

            fn is_template(&self) -> bool {
                #is_template
            }

            fn get_event_signature(&self) -> String {
                #contract_name::#event_name::SIGNATURE.to_string()
            }
        }
    })
}
