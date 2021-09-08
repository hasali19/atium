use proc_macro::TokenStream;
use quote::quote;

/// Convenience macro for writing a slightly nicer endpoint function.
///
/// This allows writing an endpoint as:
/// ```
/// #[endpoint]
/// async fn my_endpoint(req: &mut Request) -> Result<(), MyError> {
///     // Do stuff with req ...
///     Ok(())
/// }
/// ```
/// which is transformed into:
/// ```
/// async fn my_endpoint(mut req: Request) -> Request {
///     async fn my_endpoint(req: &mut Request) -> Result<(), MyError> {
///         // Do stuff with req ...
///         Ok(())
///     }
///
///     if let Err(e) = my_endpoint(&mut req).await {
///         req.set_ext(e);
///     }
///
///     req
/// }
/// ```
/// The error can then be accessed normally from the request extensions in an error handler
/// higher up in the request pipeline.
#[proc_macro_attribute]
pub fn endpoint(_: TokenStream, item: TokenStream) -> TokenStream {
    let input: syn::ItemFn = syn::parse(item).unwrap();
    let vis = input.vis.clone();
    let name = input.sig.ident.clone();

    TokenStream::from(quote! {
        #vis async fn #name(mut req: Request) -> Request {
            #input

            if let Err(e) = #name(&mut req).await {
                req.set_ext(e);
            }

            req
        }
    })
}
