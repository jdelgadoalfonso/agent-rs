use actix_web::{error::ErrorInternalServerError, HttpRequest, web::Json, Result};
use influx_db_client::{Client, Series};

pub fn index(_req: HttpRequest) -> Result<Json<Option<Vec<Series>>>> {
    let mut client = Client::default().set_authentication("root", "root");
    client.switch_database("agent-rs");
    match client.query("select * from StatSta", None) {
        Ok(r) => {
            if let Some(n) = r {
                let node = n[0].clone();
                return Ok(Json(node.series));
            }
            return Ok(Json(None));
        }
        Err(e) => Err(ErrorInternalServerError(e)),
    }
}
