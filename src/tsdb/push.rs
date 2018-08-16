use crate::parser::StatSta;
use influx_db_client::{Client, Error, Point};


pub trait Influx {
    fn push(self) -> Point;
}

pub fn push(stat_sta: StatSta) -> Result<(), Error> {
    let mut client = Client::default().set_authentication("root", "root");
    client.switch_database("udp");
    let _ = client.create_database(client.get_db().as_str())?;
    let point = stat_sta.push();
    let _ = client.write_point(point, None, None)?;

    Ok(())
}
