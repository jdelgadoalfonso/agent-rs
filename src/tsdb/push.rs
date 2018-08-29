use crate::parser::{CHTHeader, StatSta};
use influx_db_client::{Client, Error, Point, Value};

pub trait Influx {
    fn push(&self) -> Point;
}

pub fn push(cht: &CHTHeader, stat_sta: &StatSta) -> Result<(), Error> {
    let mut client = Client::default().set_authentication("root", "root");
    client.switch_database("agent-rs");
    let _ = client.create_database(client.get_db().as_str())?;
    let mut point = stat_sta.push();
    point.add_field("cht_id", Value::Integer(cht.cht_id as i64));
    let _ = client.write_point(point, None, None)?;

    Ok(())
}
