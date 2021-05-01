//! Asynchronous implementation of automatic pagination requests.

use crate::model::Page;
use crate::ClientResult;
use futures::future::Future;
use futures::stream::Stream;

/// Alias for `futures::stream::Stream<Item = T>`, since async mode is enabled.
pub type Paginator<'a, T> = Pin<Box<dyn Stream<Item = T> + 'a>>;

/// This is used to handle paginated requests automatically.
pub fn paginate<'a, T, Fut, Request>(req: Request, page_size: u32) -> Paginator<'a, T>
where
    T: Unpin,
    Fut: Future<Output = ClientResult<Page<T>>>,
    Request: Fn(u32, u32) -> Fut,
{
    use async_stream::stream;
    let mut offset = 0;
    Box::pin(stream! {
        loop {
            let page = req(page_size, offset).await?;
            offset += page.items.len() as u32;
            for item in page.items {
                yield Ok(item);
            }
            if page.next.is_none() {
                break;
            }
        }
    })
}
