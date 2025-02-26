use std::convert::{TryFrom, TryInto};

use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parenthesized, Token};

use crate::internal::*;

const OBAKE: &str = "obake";

impl Parse for VersionAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let version_str = input.parse::<syn::LitStr>()?;
        let span = version_str.span();
        let version = Version::parse(&version_str.value())
            .map_err(|err| syn::Error::new(version_str.span(), err))?;

        Ok(Self { version, span })
    }
}

impl Parse for CfgAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let req_str = input.parse::<syn::LitStr>()?;
        let span = req_str.span();
        let req = VersionReq::parse(&req_str.value())
            .map_err(|err| syn::Error::new(req_str.span(), err))?;

        Ok(Self { req, span })
    }
}

impl Parse for ObakeAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<syn::Ident>()?;

        Ok(match ident {
            _ if ident == "version" => {
                let content;
                parenthesized!(content in input);
                Self::Version(content.parse()?)
            }
            _ if ident == "cfg" => {
                let content;
                parenthesized!(content in input);
                Self::Cfg(content.parse()?)
            }
            _ if ident == "inherit" => Self::Inherit(InheritAttr { span: ident.span() }),
            _ if ident == "derive" => {
                let content;
                parenthesized!(content in input);
                Self::Derive(DeriveAttr {
                    span: ident.span(),
                    tokens: content.parse()?,
                })
            }
            _ => {
                return Err(syn::Error::new(
                    ident.span(),
                    "unrecognised `obake` helper attribute",
                ))
            }
        })
    }
}

impl TryFrom<syn::Attribute> for ObakeAttribute {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self> {
        attr.parse_args()
    }
}

impl TryFrom<syn::Attribute> for VersionedAttribute {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self> {
        attr.path.get_ident().map_or_else(
            || Ok(Self::Attribute(attr.clone())),
            |ident| {
                if ident == OBAKE {
                    Ok(Self::Obake(attr.clone().try_into()?))
                } else {
                    Ok(Self::Attribute(attr.clone()))
                }
            },
        )
    }
}

impl Parse for VersionedAttributes {
    fn parse(input: ParseStream) -> Result<VersionedAttributes> {
        let attrs = input
            .call(syn::Attribute::parse_outer)?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { attrs })
    }
}

impl Parse for VersionedField {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.parse()?,
            vis: input.parse()?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

impl Parse for VersionedFields {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let brace_token = braced!(content in input);

        Ok(Self {
            brace_token,
            fields: content.parse_terminated(VersionedField::parse)?,
        })
    }
}

impl Parse for VersionedVariantFields {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self::Unit);
        }

        let lookahead = input.lookahead1();
        Ok(if lookahead.peek(syn::token::Paren) {
            Self::Unnamed(input.parse()?)
        } else if lookahead.peek(syn::token::Brace) {
            Self::Named(input.parse()?)
        } else {
            Self::Unit
        })
    }
}

impl Parse for VersionedVariant {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.parse()?,
            ident: input.parse()?,
            fields: input.parse()?,
        })
    }
}

impl Parse for VersionedVariants {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let brace_token = braced!(content in input);

        Ok(Self {
            brace_token,
            variants: content.parse_terminated(VersionedVariant::parse)?,
        })
    }
}

impl Parse for VersionedStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            struct_token: input.parse()?,
            ident: input.parse()?,
            fields: input.parse()?,
        })
    }
}

impl Parse for VersionedEnum {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            enum_token: input.parse()?,
            ident: input.parse()?,
            variants: input.parse()?,
        })
    }
}

impl Parse for VersionedItemKind {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            Ok(Self::Struct(input.parse()?))
        } else if lookahead.peek(Token![enum]) {
            Ok(Self::Enum(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for VersionedItem {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.parse()?,
            vis: input.parse()?,
            kind: input.parse()?,
        })
    }
}
