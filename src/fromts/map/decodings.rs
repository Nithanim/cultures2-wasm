#![allow(non_camel_case_types)]
use web_sys::Blob;
use crate::fromts::map::decoding_functions::{CommonDecoded, CommonDecoded2, RawDecoded, HoixzislData};
pub use crate::fromts::map::decoding_functions::{common_decoding, common_decoding2, dictionary, raw, hoixzisl_parse};

pub trait Parser {
    type Output;

    async fn parse(&self, blob: Blob) -> std::io::Result<Self::Output>;
}

fn test(blob: Blob) {
    MapSectionName::hoixehml.parse(blob).unwrap();
}

macro_rules! impl_parser {
    ($en:path, $func:path, $out:ty) => {

        impl Parser for $en {
            type Output = $out;
            fn parse(&self, data: Blob) -> std::io::Result<Self::Output> {
                $func(data)
            }
        }
    }
}

macro_rules! create_parsing_function {
    ($variant:ident, $func:path, $out:ty) => {

        /*pub fn $variant (&self, data: &mut Cursor<Vec<u8>>) -> std::io::Result<$out> {
            $func(data)
        }*/
        pub use $func as $variant;
    }
}


macro_rules! define_map_and_enum {
    ($enum_name:ident, $($variant:ident => $func:path => $out:ty),*) => {
        // Enum variants are not type in rust and so cannot independently implement traits.
        // However, every variant has a different output, so we need that.
        // So we stretch the impl a bit to use structs with the same name as the enum and impl it there.
        // https://stackoverflow.com/questions/51567350/can-traits-be-used-on-enum-types
        /*
        $(
            #[derive(Eq, PartialEq, Hash)]
            struct $variant {

            }
            impl_parser!($variant, $func, $out);
        )*
        */

        #[derive(Eq, PartialEq, Hash, Copy)]
        pub enum $enum_name {
            $($variant),*
        }

        impl $enum_name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant)),*
                }
            }

            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    $(stringify!($variant) => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }

        $(
            create_parsing_function!($variant, $func, $out);
        )*
    }
}

define_map_and_enum!(MapSectionName,
    hoixehml => common_decoding => CommonDecoded,
    hoixapml => common_decoding => CommonDecoded,
    hoixbpml => common_decoding => CommonDecoded,
    hoixtlml => common_decoding => CommonDecoded,
    hoixvlml => common_decoding => CommonDecoded,
    hoixplml => common_decoding => CommonDecoded,
    hoixocml => common_decoding => CommonDecoded,
    hoixwtml => common_decoding => CommonDecoded,
    hoixsmml => common_decoding => CommonDecoded,
    hoixrpml => common_decoding => CommonDecoded,
    hoixbwml => common_decoding => CommonDecoded,
    hoixbbml => common_decoding => CommonDecoded,
    hoixorml => common_decoding => CommonDecoded,
    hoixbsml => common_decoding => CommonDecoded,
    hoixoaml => common_decoding2 => CommonDecoded2,
    // hoixfhml => common_decoding,
    // hoixocal => common_decoding2,
    hoixrbme => common_decoding => CommonDecoded,
    hoix1mme => common_decoding => CommonDecoded,
    hoiximme => common_decoding => CommonDecoded,

    hoixdpae => dictionary => Vec<String>,
    hoixapme => common_decoding2 => CommonDecoded2,
    hoixbpme => common_decoding2 => CommonDecoded2,

    hoixdtae => dictionary => Vec<String>,
    hoix1tme => common_decoding => CommonDecoded,
    hoix2tme => common_decoding => CommonDecoded,
    hoix3tme => common_decoding => CommonDecoded,
    hoix4tme => common_decoding => CommonDecoded,

    hoixdlae => dictionary => Vec<String>,
    hoixalme => raw => RawDecoded,
    hoixzisl => hoixzisl_parse => HoixzislData
);
#[derive(Eq, PartialEq, Hash)]
pub struct Hoixzisl {}
