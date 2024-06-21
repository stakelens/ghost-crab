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
    network: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataSource {
    abi: String,
    address: String,
    start_block: u32,
    network: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    data_sources: HashMap<String, DataSource>,
    templates: HashMap<String, Template>,
    networks: HashMap<String, String>,
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

    let content = fs::read_to_string("./config.json");

    let mut abi = String::new();
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
    let contract_name = format_ident!("{}Contract", fn_name);

    let data_source = Literal::string(&name);

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

        #[async_trait]
        impl Handler for #fn_name {
            async fn handle(&self, context: Context) {
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
