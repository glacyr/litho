//! Copa implements the [GraphQL **Co**nnection
//! **Pa**ttern](https://relay.dev/graphql/connections.htm). While mainly used
//! by GraphQL, this library is generalized to apply to any navigable data
//! source. The goal of this library is to standardize an interoperable way to
//! paginate data from a data source (e.g. a database) of a server in a way that
//! is transparent to a client. Specifically: a [`Pagination`] instance can be
//! send all the way from a client through the server to the data source, and a
//! [`Connection<T>`] instance (including its corresponding [`PageInfo`]) is
//! returned by the data source and sent all the way back through the server to
//! the client.
//!
//! ## Use Your Own Types
//! The purpose of this library is to be highly customizable.
//!
//! ### Nodes
//! Naturally, you have to define your own node type. The type is `T` in
//! [`Edge<T>`].
//!
//! ### Edges
//! A common use case for customizing the edge type is to add additional
//! metadata to an edge. The following examples introduces a new field called
//! `friends_since` to the edge.
//!
//! ```rust
//! # use chrono::{DateTime, Utc};
//! # use copa::{Connection, Cursor};
//! #
//! # pub struct User;
//! #
//! pub struct FriendshipEdge {
//!     pub node: User,
//!     pub cursor: String,
//!     pub friends_since: DateTime<Utc>,
//! }
//!
//! impl Cursor for FriendshipEdge {
//!     fn cursor(&self) -> String {
//!         self.cursor.to_owned()
//!     }
//! }
//!
//! impl User {
//!     fn friends(&self) -> Connection<FriendshipEdge> {
//!         todo!("...")
//!     }
//! }
//! ```
//!
//! ### Cursors
//! You can't bring your own cursors. Cursors are supposed to be opaque strings.
//! This usually is already the case, but if your cursors aren't strings, try
//! serializing and/or base64 encoding them.
//!
//! ### [`PageInfo`]
//! You can't customize the [`PageInfo`] object. According to the spec, no other
//! fields should be added to the [`PageInfo`] object. If you want to provide
//! more information (like the total number of results), customize the
//! [`Connection<T>`] instead (see above).
//!
//! ## Security
//! Cursors are opaque and [`copa`](crate) __does not perform any validation__.
//! It is the developer's responsibility to properly validate and sanitize any
//! cursors before passing them to e.g. the underlying database.
#![deny(missing_docs)]

use std::future::Future;

/// These are the arguments that are given to a paginated function.
#[derive(Debug, PartialEq, Eq)]
pub enum Pagination {
    /// Represents forward pagination (i.e. retrieving the first few items after
    /// the given cursor).
    Forward {
        /// Number of items to retrieve.
        first: usize,

        /// Optional exclusive start cursor.
        after: Option<String>,
    },

    /// Represents backward pagination (i.e. retrieving the last few items
    /// before the given cursor). The last item will be the one that's closest
    /// to the given cursor.
    Backward {
        /// Number of items to retrieve.
        last: usize,

        /// Optional exclusive end cursor.
        before: Option<String>,
    },
}

impl Pagination {
    /// Returns a new pagination object based on the given arguments.
    pub fn new(is_forward: bool, count: usize, cursor: Option<String>) -> Pagination {
        match is_forward {
            true => Pagination::Forward {
                first: count,
                after: cursor,
            },
            false => Pagination::Backward {
                last: count,
                before: cursor,
            },
        }
    }

    /// Returns true if the receiver requests forward pagination.
    pub fn is_forward(&self) -> bool {
        matches!(self, Pagination::Forward { .. })
    }

    /// Returns the number of items to fetch.
    pub fn count(&self) -> usize {
        match self {
            Pagination::Forward { first, .. } => *first,
            Pagination::Backward { last, .. } => *last,
        }
    }

    /// Returns the optional exclusive start or end cursor of this pagination
    /// request.
    pub fn cursor(&self) -> Option<&str> {
        match self {
            Pagination::Forward { after, .. } => after.as_deref(),
            Pagination::Backward { before, .. } => before.as_deref(),
        }
    }

    /// Returns a boolean that indicates if there may be a previous page. This
    /// doesn't necessarily mean that a non-empty page actually exists. It just
    /// means that based on these pagination arguments, the existence of a
    /// previous page cannot be ruled out.
    pub fn has_previous_page(&self) -> bool {
        match self.is_forward() {
            true => self.cursor().is_some(),
            false => true,
        }
    }

    /// Returns a boolean that indicates if there may be a next page. This
    /// doesn't necessarily mean that a non-empty page actually exists. It just
    /// means that based on these pagination arguments, the existence of a next
    /// page cannot be ruled out.
    pub fn has_next_page(&self) -> bool {
        match !self.is_forward() {
            true => self.cursor().is_some(),
            false => true,
        }
    }
}

/// Pagination information that can be used by clients to find out if there are
/// previous or next pages and what cursors to send to the server on subsequent
/// requests.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PageInfo {
    /// Indicates if there may be a previous page. This doesn't necessarily mean
    /// that a non-empty page actually exists. It just means that based on the
    /// pagination procedure, the existence of a previous page cannot be ruled
    /// out.
    pub has_previous_page: bool,

    /// Indicates if there may be a next page. This doesn't necessarily mean
    /// that a non-empty page actually exists. It just means that based on the
    /// pagination procedure, the existence of a previous page cannot be ruled
    /// out.
    pub has_next_page: bool,

    /// Contains the cursor that should be used to query the previous page. Note
    /// that the cursor may be empty even if there is a previous page. This
    /// happens when the current page is beyond the last page.
    pub start_cursor: Option<String>,

    /// Contains the cursor that should be used to query the next page. Note
    /// that the cursor may be empty even if there is a next page. This happens
    /// when the current page is before the first page.
    pub end_cursor: Option<String>,
}

impl PageInfo {
    /// Returns a new empty page info object based on the given pagination
    /// arguments.
    pub fn new(pagination: &Pagination) -> PageInfo {
        PageInfo {
            has_previous_page: pagination.has_previous_page(),
            has_next_page: pagination.has_next_page(),
            start_cursor: None,
            end_cursor: None,
        }
    }

    /// Updates the page info based on the given cursor.
    pub fn update(self, is_forward: bool, cursor: Option<String>) -> PageInfo {
        match is_forward {
            true => PageInfo {
                has_previous_page: self.has_previous_page,
                has_next_page: cursor.is_some(),
                start_cursor: self.start_cursor,
                end_cursor: cursor,
            },
            false => PageInfo {
                has_previous_page: cursor.is_some(),
                has_next_page: self.has_next_page,
                start_cursor: cursor,
                end_cursor: self.end_cursor,
            },
        }
    }
}

/// Should be implemented by edge `T` of [`Connection<T>`] and is responsible
/// for returning an opaque cursor for each edge in a connection. For basic
/// usage, we have already implemented [`Cursor`] for [`Edge<T>`].
pub trait Cursor {
    /// Should return an opaque cursor (e.g. base64-encoded) for a particular
    /// edge.
    fn cursor(&self) -> String;
}

/// Type that contains the edges and pagination information of a GraphQL-like
/// connection.
#[derive(Debug, PartialEq, Eq)]
pub struct Connection<T> {
    /// Contains the edges for this connection. The algorithm will always try to
    /// fill this with as many edges as requested or until the data source is
    /// exhausted.
    pub edges: Vec<T>,

    /// Contains information that can be sent to the client to assist in
    /// pagination (e.g. whether a previous or next page is available and
    /// cursors that should be sent back to the server).
    pub page_info: PageInfo,
}

impl<T> Connection<T> {
    /// Maps a `Connection<T>` to `Connection<U>` by applying a function to each
    /// edge in the connection.
    pub fn map<U, F>(self, f: F) -> Connection<U>
    where
        F: FnMut(T) -> U,
    {
        Connection {
            edges: self.edges.into_iter().map(f).collect(),
            page_info: self.page_info,
        }
    }

    /// Maps a `Connection<T>` to `Connection<U>` by applying a fallible
    /// function to each edge in the connection.
    pub fn try_map<U, E, F>(self, f: F) -> Result<Connection<U>, E>
    where
        F: FnMut(T) -> Result<U, E>,
    {
        Ok(Connection {
            edges: self.edges.into_iter().map(f).collect::<Result<_, _>>()?,
            page_info: self.page_info,
        })
    }

    /// Asynchronously maps a `Connection<T>` to `Connection<U>` by applying an
    /// async function to each edge in the connection.
    pub async fn then<Fut, F>(self, f: F) -> Connection<Fut::Output>
    where
        F: Fn(T) -> Fut,
        Fut: Future,
    {
        self.try_then(|item| async { Ok::<_, ()>(f(item).await) })
            .await
            .unwrap()
    }

    /// Asynchronously maps a `Connection<T>` to `Connection<U>` by applying a
    /// fallible async function to each edge in the connection.
    pub async fn try_then<'a, Fut, U, E, F>(self, f: F) -> Result<Connection<U>, E>
    where
        F: Fn(T) -> Fut,
        Fut: Future<Output = Result<U, E>>,
    {
        let mut edges = Vec::with_capacity(self.edges.len());

        for edge in self.edges {
            edges.push(f(edge).await?);
        }

        Ok(Connection {
            edges,
            page_info: self.page_info,
        })
    }
}

/// Represents a single typed edge in a connection, along with a cursor that
/// can be used for pagination.
#[derive(Debug, PartialEq, Eq)]
pub struct Edge<T> {
    /// Contains the node that this edge points to.
    pub node: T,

    /// Contains a cursor that can be used to retrieve adjacent edges.
    pub cursor: String,
}

impl<T> Cursor for Edge<T> {
    fn cursor(&self) -> String {
        self.cursor.to_owned()
    }
}

/// Return type of a data source passed to [`paginate`].
pub struct ResultSet<T> {
    /// Contains the edges that this invocation to the data source yielded. Note
    /// that this may contain more or fewer edges than requested. The algorithm
    /// will make to sure to keep invoking the data source if the result set
    /// contains fewer edges than requested, or will truncate the result set if
    /// it contains more edges than requested. Note that the edges must always
    /// appear in the same order (regardless of [`Pagination::is_forward`]).
    pub edges: Vec<T>,

    /// Contains the cursor of the last evaluated edge. Note that this may not
    /// necessarily be the last returned edge (e.g. when using filters for
    /// authorization).
    pub cursor: Option<String>,
}

/// Implementation of the pagination algorithm that works with a non-cooperative
/// data source (i.e. one that sends more or fewer items than requested).
pub async fn paginate<S, F, T, E>(source: S, pagination: Pagination) -> Result<Connection<T>, E>
where
    S: Fn(Pagination) -> F,
    F: Future<Output = Result<ResultSet<T>, E>>,
    T: Cursor,
{
    let count = pagination.count();

    let mut edges = vec![];
    let mut cursor = pagination.cursor().map(ToOwned::to_owned);
    let page_info = PageInfo::new(&pagination);
    let is_forward = pagination.is_forward();

    while edges.len() < count {
        let pagination = Pagination::new(is_forward, count - edges.len(), cursor);

        let set = source(pagination).await?;

        if is_forward {
            edges.extend(set.edges);
            edges.truncate(count);
        } else {
            let len = set.edges.len() + edges.len();
            let skip = if len > count { len - count } else { 0 };
            edges = set
                .edges
                .into_iter()
                .chain(edges.into_iter())
                .skip(skip)
                .collect();
        }

        cursor = set.cursor;

        if cursor.is_none() {
            break;
        }
    }

    let mut page_info = page_info.update(is_forward, cursor);

    match is_forward {
        true => page_info.start_cursor = edges.first().map(Cursor::cursor),
        false => page_info.end_cursor = edges.last().map(Cursor::cursor),
    }

    Ok(Connection { edges, page_info })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ops::Bound;

    use futures::FutureExt;

    use super::{paginate, Connection, Cursor, PageInfo, Pagination, ResultSet};

    impl Cursor for String {
        fn cursor(&self) -> String {
            self.to_owned()
        }
    }

    async fn dummy_data_source(pagination: Pagination) -> Result<ResultSet<String>, ()> {
        let btree = BTreeMap::from_iter(vec![
            ("a".to_owned(), ()),
            ("b".to_owned(), ()),
            ("c".to_owned(), ()),
            ("d".to_owned(), ()),
            ("e".to_owned(), ()),
            ("f".to_owned(), ()),
            ("g".to_owned(), ()),
        ]);

        let is_forward = pagination.is_forward();

        let mut edges: Vec<_> = match pagination {
            Pagination::Forward { first, after } => btree
                .range((
                    match after {
                        Some(after) => Bound::Excluded(after),
                        None => Bound::Unbounded,
                    },
                    Bound::Unbounded,
                ))
                .map(|(a, _)| a)
                .take(first)
                .take(2)
                .cloned()
                .collect(),
            Pagination::Backward { last, before } => btree
                .range((
                    Bound::Unbounded,
                    match before {
                        Some(before) => Bound::Excluded(before),
                        None => Bound::Unbounded,
                    },
                ))
                .map(|(a, _)| a)
                .rev()
                .take(last)
                .take(2)
                .cloned()
                .collect(),
        };

        let cursor = edges.last().map(ToOwned::to_owned);

        if !is_forward {
            edges.reverse();
        }

        Ok::<_, ()>(ResultSet { edges, cursor })
    }

    #[test]
    fn test_first_page() {
        assert_eq!(
            paginate(
                dummy_data_source,
                Pagination::Forward {
                    first: 2,
                    after: None
                }
            )
            .now_or_never()
            .unwrap(),
            Ok(Connection {
                edges: vec!["a".to_owned(), "b".to_owned()],
                page_info: PageInfo {
                    has_next_page: true,
                    start_cursor: Some("a".to_owned()),
                    end_cursor: Some("b".to_owned()),
                    ..Default::default()
                }
            })
        );
    }

    #[test]
    fn test_first_pages() {
        assert_eq!(
            paginate(
                dummy_data_source,
                Pagination::Forward {
                    first: 3,
                    after: None
                }
            )
            .now_or_never()
            .unwrap(),
            Ok(Connection {
                edges: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
                page_info: PageInfo {
                    has_next_page: true,
                    start_cursor: Some("a".to_owned()),
                    end_cursor: Some("c".to_owned()),
                    ..Default::default()
                }
            })
        );
    }

    #[test]
    fn test_second_page() {
        assert_eq!(
            paginate(
                dummy_data_source,
                Pagination::Forward {
                    first: 3,
                    after: Some("c".to_owned())
                }
            )
            .now_or_never()
            .unwrap(),
            Ok(Connection {
                edges: vec!["d".to_owned(), "e".to_owned(), "f".to_owned()],
                page_info: PageInfo {
                    has_previous_page: true,
                    has_next_page: true,
                    start_cursor: Some("d".to_owned()),
                    end_cursor: Some("f".to_owned()),
                    ..Default::default()
                }
            })
        );
    }

    #[test]
    fn test_forward_empty() {
        assert_eq!(
            paginate(
                dummy_data_source,
                Pagination::Forward {
                    first: 3,
                    after: Some("g".to_owned())
                }
            )
            .now_or_never()
            .unwrap(),
            Ok(Connection {
                edges: vec![],
                page_info: PageInfo {
                    has_previous_page: true,
                    has_next_page: false,
                    start_cursor: None,
                    end_cursor: None,
                    ..Default::default()
                }
            })
        );
    }

    #[test]
    fn test_forward_half_empty() {
        assert_eq!(
            paginate(
                dummy_data_source,
                Pagination::Forward {
                    first: 3,
                    after: Some("f".to_owned())
                }
            )
            .now_or_never()
            .unwrap(),
            Ok(Connection {
                edges: vec!["g".to_owned()],
                page_info: PageInfo {
                    has_previous_page: true,
                    has_next_page: false,
                    start_cursor: Some("g".to_owned()),
                    end_cursor: None,
                    ..Default::default()
                }
            })
        );
    }

    #[test]
    fn test_last_page() {
        assert_eq!(
            paginate(
                dummy_data_source,
                Pagination::Backward {
                    last: 2,
                    before: None
                }
            )
            .now_or_never()
            .unwrap(),
            Ok(Connection {
                edges: vec!["f".to_owned(), "g".to_owned()],
                page_info: PageInfo {
                    has_previous_page: true,
                    start_cursor: Some("f".to_owned()),
                    end_cursor: Some("g".to_owned()),
                    ..Default::default()
                }
            })
        );
    }

    #[test]
    fn test_last_pages() {
        assert_eq!(
            paginate(
                dummy_data_source,
                Pagination::Backward {
                    last: 3,
                    before: None
                }
            )
            .now_or_never()
            .unwrap(),
            Ok(Connection {
                edges: vec!["e".to_owned(), "f".to_owned(), "g".to_owned()],
                page_info: PageInfo {
                    has_previous_page: true,
                    start_cursor: Some("e".to_owned()),
                    end_cursor: Some("g".to_owned()),
                    ..Default::default()
                }
            })
        );
    }

    #[test]
    fn test_second_last_page() {
        assert_eq!(
            paginate(
                dummy_data_source,
                Pagination::Backward {
                    last: 3,
                    before: Some("f".to_owned())
                }
            )
            .now_or_never()
            .unwrap(),
            Ok(Connection {
                edges: vec!["c".to_owned(), "d".to_owned(), "e".to_owned()],
                page_info: PageInfo {
                    has_previous_page: true,
                    has_next_page: true,
                    start_cursor: Some("c".to_owned()),
                    end_cursor: Some("e".to_owned()),
                    ..Default::default()
                }
            })
        );
    }

    #[test]
    fn test_backward_empty() {
        assert_eq!(
            paginate(
                dummy_data_source,
                Pagination::Backward {
                    last: 3,
                    before: Some("a".to_owned())
                }
            )
            .now_or_never()
            .unwrap(),
            Ok(Connection {
                edges: vec![],
                page_info: PageInfo {
                    has_previous_page: false,
                    has_next_page: true,
                    start_cursor: None,
                    end_cursor: None,
                    ..Default::default()
                }
            })
        );
    }

    #[test]
    fn test_backward_half_empty() {
        assert_eq!(
            paginate(
                dummy_data_source,
                Pagination::Backward {
                    last: 3,
                    before: Some("b".to_owned())
                }
            )
            .now_or_never()
            .unwrap(),
            Ok(Connection {
                edges: vec!["a".to_owned()],
                page_info: PageInfo {
                    has_previous_page: false,
                    has_next_page: true,
                    start_cursor: None,
                    end_cursor: Some("a".to_owned()),
                    ..Default::default()
                }
            })
        );
    }
}
