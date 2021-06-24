mod lib;
use lib::Post;

fn main() {
    let mut post = Post::new();

    post.add_text("I ate a salad for lunch today");
    assert_eq!("", post.content());

    post.request_review();
    assert_eq!("", post.content());
    post.reject();
    post.request_review();

    post.approve();
    post.approve();
    assert_eq!("I ate a salad for lunch today", post.content());
}
