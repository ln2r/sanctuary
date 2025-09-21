use worker::*;
use uuid::{NoContext, Timestamp, Uuid};
use worker::wasm_bindgen::JsValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Image {
    id: String,
    created: String,
    updated: String,
    deleted: String,
    captured: String,
    published: String,
    path: String,
    caption: String,
    views: u32,
}

#[event(fetch)]
async fn fetch(
    req: Request,
    env: Env,
    _ctx: Context,
) -> Result<Response> {
    let router = Router::new();

    router
        .post_async("/upload", |mut req, ctx| async move {
            // this is to check key
            // TODO: move as function guard
            let request_key = req.headers().get("x-sanctuary-key")?.unwrap_or_default();
            if request_key != ctx.env.var("api_key")?.to_string() {
                return Response::error("Unauthorized", 401)
            }

            let d1 = ctx.env.d1("DB")?;
            let bucket = ctx.env.bucket("STORAGE")?;

            let form = req.form_data().await?;

            if let Some(FormEntry::File(file)) = form.get("image") {
                let name = file.name().to_string();
                let bytes = file.bytes().await?;

                // uploading to r2
                let obj = bucket
                    .put(&name, bytes)
                    .http_metadata(HttpMetadata {
                        content_type: Some(file.type_().to_string()),
                        ..Default::default()
                    })
                    .execute()
                    .await?;

                let ts = Timestamp::from_unix(NoContext, 1497624119, 1234);
                let id = Uuid::new_v7(ts);
                let created = obj.uploaded();
                let captured = file.last_modified();
                let path = obj.key();
                let caption = match form.get("caption") { // getting the caption key
                    Some(FormEntry::Field(value)) => Some(value.clone()),
                    _ => None, // set to null
                };

                d1.prepare("INSERT INTO images (id, created, updated, captured, path, caption) VALUES (?, ?, ?, ?, ?);")
                    .bind(&[
                        JsValue::from(&id.to_string()),
                        JsValue::from(&created.to_string()),
                        JsValue::from(&created.to_string()),
                        JsValue::from(&captured.to_string()),
                        JsValue::from(&path),
                        match caption {
                            Some(ref s) => JsValue::from(s), // handling null
                            None => JsValue::NULL,
                        },
                    ])?
                    .run()
                    .await?;
            }

            Response::ok("Success")
        })
        .delete_async("/delete/:id", |req, ctx| async move {
            // this is to check key
            // TODO: move as function guard
            let request_key = req.headers().get("x-sanctuary-key")?.unwrap_or_default();
            if request_key != ctx.env.var("api_key")?.to_string() {
                return Response::error("Unauthorized", 401)
            }

            let d1 = ctx.env.d1("DB")?;
            let post_id = ctx.param("id").unwrap().to_string();

            let query = d1.prepare("SELECT * FROM images WHERE id = ?")
                .bind(&[JsValue::from(&post_id)]);

            let exist = query?.first::<Image>(None).await?;
            console_log!("{:?}", exist);
            match exist {
                Some(image) => {
                    console_log!("{:?}", image);
                    Response::from_json(&image)
                    
                },
                None => {
                    console_log!("not found");
                    Response::error("Not found", 404)
                },
            }.unwrap();

            Response::ok("Success")
        })
        .patch_async("/update/:id", |mut req, ctx| async move {
            // this is to check key
            // TODO: move as function guard
            let request_key = req.headers().get("x-sanctuary-key")?.unwrap_or_default();
            if request_key != ctx.env.var("api_key")?.to_string() {
                return Response::error("Unauthorized", 401)
            }

            // let d1 = ctx.env.d1("DB")?;
            // let post_id = ctx.param("id").unwrap().to_string();

            Response::ok("Success")
        })
        .run(req, env)
        .await
}