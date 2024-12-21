use curies::{Converter, Record};
use log::error;
use tree_sitter::Parser;

pub(super) struct Tools {
    pub(super) uri_converter: Converter,
    pub(super) parser: Parser,
}

impl Tools {
    pub(super) fn initiate() -> Self {
        let mut tools = Self {
            uri_converter: Converter::new(":"),
            parser: Parser::new(),
        };

        tools.initiate_uri_converter();
        tools.initiate_parser();
        return tools;
    }

    fn initiate_parser(&mut self) {
        match self
            .parser
            .set_language(&tree_sitter_sparql::LANGUAGE.into())
        {
            Ok(()) => {}
            Err(err) => error!("Error while initializing parser: {}", err),
        }
    }

    fn initiate_uri_converter(&mut self) {
        let records = [
            Record::new("schema", "http://schema.org/"),
            Record::new(
                "envCube",
                "https://environment.ld.admin.ch/foen/nfi/nfi_C-20/cube/",
            ),
            Record::new(
                "envTopic",
                "https://environment.ld.admin.ch/foen/nfi/Topic/",
            ),
            Record::new("cube", "https://cube.link/"),
            Record::new("env", "https://environment.ld.admin.ch/foen/nfi/"),
            Record::new("country", "https://ld.admin.ch/country/"),
        ];
        records.into_iter().for_each(|record| {
            if let Err(error) = self.uri_converter.add_record(record.clone()) {
                error!("Could not setup custom prefix:\n{}", error);
            }
        });
    }
}