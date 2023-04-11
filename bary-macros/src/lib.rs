
use proc_macro::{TokenStream};
use quote::quote;
use syn::{parse_macro_input, Item};

#[proc_macro_attribute]
pub fn bary_app(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    let cloned_item = item.clone();
    let parsed_item = parse_macro_input!(cloned_item as Item);
    let func = match parsed_item {
        Item::Fn(func) => func,
        _ => panic!("Tried using something that's not a function for a bary app"),
    };    
    let ident = func.sig.ident;
    let main = quote!(
        fn main() {
            let frontend = bary::frontend_setup!();
            let config = bary::config::load_config_from_str(include_str!(concat!(env!("OUT_DIR"), "/bary.yaml"))).expect("Couldn't load config");
            let bary = bary::Server::new(config.port, frontend);
            #ident(bary);
        }
    );
    item.extend(TokenStream::from(main));
    item
}
