use elastic::{
    Error,
    prelude::{
        Date, DefaultDateMapping, EpochMillis, id, Index, SyncClientBuilder
    }
};

use parser::CHTHeader;

use std::convert::From;


#[derive(Serialize, Deserialize, ElasticType)]
pub struct MyType {
    // Mapped as an `integer` field
    pub id: i32,
    // Mapped as a `text` field with a `keyword` subfield
    pub title: String,
    // Mapped as a `date` field with an `epoch_millis` format
    pub timestamp: Date<DefaultDateMapping<EpochMillis>>
}

impl From<CHTHeader> for MyType {
    fn from(_f: CHTHeader) -> Self {
        // TODO: fill fields with CHTHeader info
        MyType {
            id: 0,
            title: String::from("title"),
            timestamp: Date::now(),
        }
    }
}

fn sample_index() -> Index<'static> {
    Index::from("index_sample_index")
}

pub fn push(doc: &MyType) -> Result<(), Error> {
    // A reqwest HTTP client and default parameters.
    // The builder includes the base node url (http://localhost:9200).
    let client = SyncClientBuilder::new().build()?;

    // Create the index if it doesn't exist
    if !client.index_exists(sample_index()).send()?.exists() {
        client.index_create(sample_index()).send()?;
    }

    // Add the document mapping (optional, but makes sure `timestamp`
    // is mapped as a `date`)
    client.document_put_mapping::<MyType>(sample_index())
    .send()?;

    // Index the document
    client.document_index(sample_index(), id(doc.id), doc)
    .send()?;

    Ok(())
}
