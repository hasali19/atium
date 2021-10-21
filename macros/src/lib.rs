use proc_macro::TokenStream;
use quote::quote;

/// Convenience macro for writing a slightly nicer endpoint function.
///
/// This allows writing an endpoint as:
/// ```
/// #[endpoint]
/// async fn my_endpoint(req: &mut Request) -> Result<impl Responder, MyError> {
///     Ok("hello, world!")
/// }
/// ```
/// which is transformed into something like:
/// ```
/// async fn my_endpoint(mut req: Request) -> Request {
///     async fn my_endpoint(req: &mut Request) -> Result<impl Responder, MyError> {
///         Ok("hello, world!")
///     }
///
///     my_endpoint(&mut req).await.respond_to(&mut req);
///
///     req
/// }
/// ```
/// The error can then be accessed normally from the request extensions in an error handler
/// higher up in the request pipeline.
#[proc_macro_attribute]
pub fn endpoint(_: TokenStream, mut item: TokenStream) -> TokenStream {
    let input: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(input) => input,
        Err(e) => {
            item.extend(TokenStream::from(e.into_compile_error()));
            return item;
        }
    };

    let vis = input.vis.clone();
    let name = input.sig.ident.clone();

    TokenStream::from(quote! {
        #vis async fn #name(mut req: Request) -> Request {
            #input
            let res = #name(&mut req).await;
            atium::responder::Responder::respond_to(res, &mut req);
            req
        }
    })
}
