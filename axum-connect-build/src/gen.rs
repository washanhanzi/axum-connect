use prost_build::{Service, ServiceGenerator};
use quote::{format_ident, quote};

#[derive(Default)]
pub struct AxumConnectServiceGenerator;

impl AxumConnectServiceGenerator {
    pub fn new() -> Self {
        Default::default()
    }
}

impl ServiceGenerator for AxumConnectServiceGenerator {
    fn generate(&mut self, service: Service, buf: &mut String) {
        let service_module_name = format_ident!("{}", service.name.to_lowercase());

        let handler_params = service.methods.iter().map(|method| {
            let handler_name = format_ident!("{}_handler", method.name.to_lowercase());
            quote! { #handler_name: impl axum::handler::Handler<_, S, axum::body::Body> + 'static }
        });

        let routes = service.methods.iter().map(|method| {
            let path = format!("/{}.{}/{}", service.package, service.proto_name, method.proto_name);
            let handler_name = format_ident!("{}_handler", method.name.to_lowercase());
            quote! {
                .route(
                    #path,
                    axum::routing::post(#handler_name)
                )
            }
        });

        let routes_fn = quote! {
            pub mod #service_module_name {
                pub fn routes<S: Clone + Send + Sync + 'static>(
                    #(#handler_params),*
                ) -> axum::Router<S> {
                    axum::Router::new()
                        #(#routes)*
                }
            }
        };

        buf.push_str(&routes_fn.to_string());
    }
}
