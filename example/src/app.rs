use std::sync::Arc;

use lazy_static::lazy_static;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/static/main.css"/>
        <Title text="Welcome to Leptos"/>

        <Router>
            <main>
                <Routes>
                    <Route path="/" view=HomePage />
                    <Route
                        path="/post/out-of-order/:id"
                        view=Post
                        ssr=SsrMode::OutOfOrder
                    />
                    <Route
                        path="/post/partially-blocked/:id"
                        view=Post
                        ssr=SsrMode::PartiallyBlocked
                    />
                    <Route
                        path="/post/in-order/:id"
                        view=Post
                        ssr=SsrMode::InOrder
                    />
                    <Route
                        path="/post/async/:id"
                        view=Post
                        ssr=SsrMode::Async
                    />
                    <Route
                        path="/*any"
                        view=NotFound
                    />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    // load the posts
    let posts = create_resource(|| (), |_| async { list_post_metadata().await });
    let posts_view = move || {
        posts.with(|posts| posts
            .clone()
            .map(|posts| {
                posts
                .map(|posts| {
                    posts.iter()
                    .map(|post| view! {<ul>
                        <li><a href=format!("/post/out-of-order/{}", post.id)>out of order: {&post.title}</a></li>
                        <li><a href=format!("/post/partially-blocked/{}", post.id)>partially blocked: {&post.title}</a></li>
                        <li><a href=format!("/post/in-order/{}", post.id)>in order: {&post.title}</a></li>
                        <li><a href=format!("/post/async/{}", post.id)>async: {&post.title}</a></li>
                    </ul>
                    })
                    .collect_view()
                })
            })
        )
    };

    view! {
        <h1>"My Great Blog"</h1>
        <img
            src="/static/leptos.png"
            style="width: 150px; height: 100px;"
        />
        <Suspense fallback=move || view! { <p>"Loading posts..."</p> }>
            <ul>{posts_view}</ul>
        </Suspense>
    }
}

#[derive(Params, Copy, Clone, Debug, PartialEq, Eq)]
pub struct PostParams {
    id: PostId,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct PostId(usize);

impl IntoParam for PostId {
    fn into_param(value: Option<&str>, name: &str) -> Result<Self, ParamsError> {
        match value {
            Some(value) => match value.parse::<usize>() {
                Ok(id) => Ok(PostId(id)),
                Err(err) => Err(ParamsError::Params(Arc::new(err))),
            },
            None => Err(ParamsError::MissingParam(name.to_string())),
        }
    }
}

#[component]
fn Post() -> impl IntoView {
    let query = use_params::<PostParams>();
    let id = move || query.with(|q| q.as_ref().map(|q| q.id).map_err(|_| PostError::InvalidId));
    let post = create_resource(id, |id| async move {
        match id {
            Err(e) => Err(e),
            Ok(id) => get_post(id.0)
                .await
                .and_then(|data| data.ok_or(PostError::PostNotFound.into()))
                .map_err(|_| PostError::ServerError),
        }
    });

    let post_view = move || {
        post.with(|post| {
            post.clone().map(|post| {
                post.map(|post| {
                    view! {
                        // render content
                        <h1>{&post.title}</h1>
                        <p>{&post.content}</p>

                        // since we're using async rendering for this page,
                        // this metadata should be included in the actual HTML <head>
                        // when it's first served
                        <Title text=post.title/>
                        <Meta name="description" content=post.content/>
                    }
                })
            })
        })
    };

    view! {
        <Suspense fallback=move || view! { <p>"Loading post..."</p> }>
            <ErrorBoundary fallback=|errors| {
                view! {
                    <div class="error">
                        <h1>"Something went wrong."</h1>
                        <ul>
                        {move || errors.get()
                            .into_iter()
                            .map(|(_, error)| view! { <li>{error.to_string()} </li> })
                            .collect_view()
                        }
                        </ul>
                    </div>
                }
            }>
                {post_view}
            </ErrorBoundary>
        </Suspense>
    }
}

// Dummy API
lazy_static! {
    static ref POSTS: Vec<Post> = vec![
        Post {
            id: 0,
            title: "My first post".to_string(),
            content: "This is my first post".to_string(),
        },
        Post {
            id: 1,
            title: "My second post".to_string(),
            content: "This is my second post".to_string(),
        },
        Post {
            id: 2,
            title: "My third post".to_string(),
            content: "This is my third post".to_string(),
        },
    ];
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostError {
    #[error("Invalid post ID.")]
    InvalidId,
    #[error("Post not found.")]
    PostNotFound,
    #[error("Server error.")]
    ServerError,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Post {
    id: usize,
    title: String,
    content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostMetadata {
    id: usize,
    title: String,
}

#[server(ListPostMetadata, "/api")]
pub async fn list_post_metadata() -> Result<Vec<PostMetadata>, ServerFnError> {
    Ok(POSTS
        .iter()
        .map(|data| PostMetadata {
            id: data.id,
            title: data.title.clone(),
        })
        .collect())
}

#[server(GetPost, "/api")]
pub async fn get_post(id: usize) -> Result<Option<Post>, ServerFnError> {
    Ok(POSTS.iter().find(|post| post.id == id).cloned())
}

#[component]
fn NotFound() -> impl IntoView {
    view! { <h1>"Not Found"</h1> }
}
