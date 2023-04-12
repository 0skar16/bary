
use bary_server::BaryAppAttr;
use proc_macro::{TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Item};

#[proc_macro_attribute]
pub fn bary_app(attr: TokenStream, mut item: TokenStream) -> TokenStream {
    let attr: BaryAppAttr = serde_tokenstream::from_tokenstream(&TokenStream2::from(attr)).expect("Couldn't parse bary_app attribute");

    let cloned_item = item.clone();
    let parsed_item = parse_macro_input!(cloned_item as Item);
    let func = match parsed_item {
        Item::Fn(func) => func,
        _ => panic!("Tried using something that's not a function for a bary app"),
    };    
    let key = attr.secret_key;
    let ident = func.sig.ident;
    let main = quote!(
        fn main() {
            let frontend = bary::frontend_setup!();
            let config = bary::load_config_from_str(include_str!(concat!(env!("OUT_DIR"), "/bary.yaml"))).expect("Couldn't load config");
            let bary = bary::Server::new(config.port, frontend, Some(#key));
            #ident(bary);
        }
    );
    item.extend(TokenStream::from(main));
    item
}
