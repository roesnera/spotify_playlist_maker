#[get("/")]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/index.html")).await.ok()
}

#[get("/success?<code>")]
pub async fn success(code: &str, auth_state: &State<Code>) -> Redirect {
    let mut code_state = auth_state
        .auth_code
        .lock()
        .expect("Could not lock state mutex");
    *code_state = Some(code.to_owned());
    Redirect::to(uri!(make_playlist()))
}

#[get("/makePlaylist")]
pub async fn make_playlist(code: String) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/makePlaylist.html"))
        .await
        .ok()
}

#[post("/makePlaylist")]

