#[cfg(test)]
mod tests {
    use actix_web::{body::to_bytes, dev::Service, http, test, App, Error};
    
    use crate::app_config::config_app;

    #[actix_web::test]
    async fn test_index() -> Result<(), Error> {
        let app = App::new().configure(config_app);
        let app = test::init_service(app).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = app.call(req).await?;

        assert_eq!(resp.status(), http::StatusCode::OK);

        // let response_body = resp.into_body();
        // assert_eq!(to_bytes(response_body).await?, r##"Hello world!"##);

        Ok(())
    }
}