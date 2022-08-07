use kono::AspectExt;
use kono_macros::kono;

pub struct User;

#[kono]
impl User {
    fn name(&self) -> &str {
        "Tim"
    }

    fn viewer() -> User {
        User
    }
}

#[tokio::main]
pub async fn main() {
    let (accept, process) = kono::server(User::resolver(), || ());

    futures::future::join(
        warp::serve(kono::server::warp::filter(accept)).run(([127, 0, 0, 1], 3030)),
        process,
    )
    .await;
}
