use std::borrow::Borrow;

use futures::{
    future::{BoxFuture, FutureExt},
    stream::{self, Stream},
};
use google_photoslibrary1::{api::MediaItem, PhotosLibrary};

pub use google_photoslibrary1::client::Error as ApiError;

struct State<T> {
    hub: T,
    buffer: Vec<MediaItem>,
    next_page_token: Option<String>,
    done: bool,
}

impl<'a, T> State<T>
where
    T: 'a + Borrow<PhotosLibrary> + Send,
{
    fn next(mut self) -> BoxFuture<'a, Result<Option<(MediaItem, Self)>, ApiError>> {
        // TODO: Find a way to do this that doesn't alloc quite as much
        async {
            match self.buffer.pop() {
                Some(next) => Ok(Some((next, self))),
                None => {
                    if self.done {
                        Ok(None)
                    } else {
                        let req = self.hub.borrow().media_items().list().page_size(100);
                        let req = match self.next_page_token {
                            Some(token) => req.page_token(token.as_str()),
                            None => req,
                        };
                        let result = req.doit().await;
                        let (_body, response) = result?;
                        let done = response.next_page_token.is_none();
                        let mut items = response.media_items.unwrap_or_else(|| vec![]);
                        items.reverse();

                        let new_state = State {
                            hub: self.hub,
                            buffer: items,
                            next_page_token: response.next_page_token,
                            done,
                        };

                        new_state.next().await
                    }
                }
            }
        }
        .boxed()
    }
}

pub fn list<'a, T>(hub: T) -> impl 'a + Stream<Item = Result<MediaItem, ApiError>>
where
    T: 'a + Borrow<PhotosLibrary> + Send,
{
    let initial = State {
        hub,
        buffer: vec![],
        next_page_token: None,
        done: false,
    };
    stream::try_unfold(initial, |state| async move { state.next().await })
}
