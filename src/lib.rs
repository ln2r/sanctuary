use worker::*;
use uuid::{NoContext, Timestamp, Uuid};
use worker::wasm_bindgen::JsValue;

#[event(fetch)]
async fn fetch(
    req: Request,
    env: Env,
    _ctx: Context,
) -> Result<Response> {
    let router = Router::new();

    router
        .post_async("/upload", |mut req, ctx| async move {
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

                d1.prepare("INSERT INTO images (id, created, captured, path, caption) VALUES (?, ?, ?, ?, ?);")
                    .bind(&[
                        JsValue::from(&id.to_string()),
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

            Response::ok("upload")
        })
        .run(req, env)
        .await
}