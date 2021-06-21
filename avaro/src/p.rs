use pest_consume::{Parser, Error, match_nodes};
use bigdecimal::BigDecimal;
use std::str::FromStr;
use crate::models::{AvaroString, AccountType, Directive, Account, StringOrAccount};
use chrono::NaiveDate;
use indexmap::map::IndexMap;

type Result<T> = std::result::Result<T, Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "avaro.pest"]
pub struct AvaroParser;

#[pest_consume::parser]
impl AvaroParser {
    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }
    fn number(input: Node) -> Result<BigDecimal> {
        Ok(BigDecimal::from_str(input.as_str()).unwrap())
    }
    fn inner(input: Node) -> Result<String> {
        Ok(input.as_str().to_owned())
    }
    fn QuoteString(input: Node) -> Result<AvaroString> {
        let ret: String = match_nodes!(
            input.into_children();
            [inner(i)] => i
        );
        Ok(AvaroString::QuoteString(ret))
    }
    fn UnquoteString(input: Node) -> Result<AvaroString> {
        Ok(AvaroString::UnquoteString(input.as_str().to_owned()))
    }

    fn String(input: Node) -> Result<AvaroString> {
        let ret = match_nodes!(
            input.into_children();
            [QuoteString(i)] => i,
            [UnquoteString(i)] => i
        );
        Ok(ret)
    }
    fn CommodityName(input: Node) -> Result<String> {
        Ok(input.as_str().to_owned())
    }
    fn AccountType(input: Node) -> Result<String> {
        Ok(input.as_str().to_owned())
    }
    fn AccountName(input: Node) -> Result<Account> {
        let r: (String, Vec<AvaroString>) = match_nodes!(input.into_children();
            [AccountType(a), UnquoteString(i)..] => {
                (a, i.collect())
            },

        );
        Ok(Account {
            account_type: AccountType::from_str(&r.0).unwrap(),
            value: r.1.into_iter().map(|it| it.to_string()).collect(),
        })
    }
    fn Date(input: Node) -> Result<NaiveDate> {
        Ok(NaiveDate::parse_from_str(input.as_str(), "%Y-%m-%d").unwrap())
    }


    fn Plugin(input: Node) -> Result<Directive> {
        let ret: (AvaroString, Vec<AvaroString>) = match_nodes!(input.into_children();
            [String(module), String(values)..] => (module, values.collect()),
        );
        let values = ret.1.into_iter().map(|it| it.to_string()).collect();
        Ok(Directive::Plugin { module: ret.0.to_string(), value: values })
    }

    fn Option(input: Node) -> Result<Directive> {
        let ret = match_nodes!(input.into_children();
            [String(key), String(value)] => Directive::Option {key:key.to_string(),value:value.to_string()},
        );
        Ok(ret)
    }

    fn Open(input: Node) -> Result<Directive> {
        let ret: (NaiveDate, Account, Vec<String>) = match_nodes!(input.into_children();
            [Date(date), AccountName(a), CommodityName(commodities)..] => (date, a, commodities.collect())
        );
        Ok(Directive::Open {
            date: ret.0,
            account: ret.1,
            commodities: ret.2,
        })
    }
    fn Close(input: Node) -> Result<Directive> {
        let ret: (NaiveDate, Account) = match_nodes!(input.into_children();
            [Date(date), AccountName(a)] => (date, a)
        );
        Ok(Directive::Close {
            date: ret.0,
            account: ret.1,
        })
    }

    fn identation(input: Node) -> Result<()> { Ok(()) }

    fn CommodityLine(input: Node) -> Result<(AvaroString, AvaroString)> {
        let ret: (AvaroString, AvaroString) = match_nodes!(input.into_children();
            [String(key), String(value)] => (key, value),
        );
        Ok(ret)
    }
    fn CommodityNextLine(input: Node) -> Result<(AvaroString, AvaroString)> {
        let ret: (AvaroString, AvaroString) = match_nodes!(input.into_children();
            [identation(_), CommodityLine(line)] => line,
        );
        Ok(ret)
    }
    fn CommodityLines(input: Node) -> Result<Vec<(AvaroString, AvaroString)>> {
        let ret: Vec<(AvaroString, AvaroString)> = match_nodes!(input.into_children();
            [CommodityLine(line)] => vec![line],
            [CommodityLine(line), CommodityNextLine(lines)..] => {
                let mut vec: Vec<(AvaroString, AvaroString)> = lines.collect();
                vec.insert(0, line);
                vec
            },
        );
        Ok(ret)
    }
    fn CommodityMeta(input: Node) -> Result<Vec<(AvaroString, AvaroString)>> {
        let ret: Vec<(AvaroString, AvaroString)> = match_nodes!(input.into_children();
            [identation(_), CommodityLines(lines)] => lines,
        );
        Ok(ret)
    }

    fn Commodity(input: Node) -> Result<Directive> {
        let ret = match_nodes!(input.into_children();
            [Date(date), CommodityName(name)] => (date, name, vec![]),
            [Date(date), CommodityName(name), CommodityMeta(meta)] => (date, name, meta),
        );
        Ok(Directive::Commodity {
            date: ret.0,
            name: ret.1,
            metas: ret.2,
        })
    }

    fn StringOrAccount(input:Node) ->Result<StringOrAccount> {
        let ret: StringOrAccount = match_nodes!(input.into_children();
            [String(value)] => StringOrAccount::String(value),
            [AccountName(value)] => StringOrAccount::Account(value),
        );
        Ok(ret)
    }

    fn Custom(input:Node) -> Result<Directive> {
        let ret: (NaiveDate, AvaroString, Vec<StringOrAccount>) = match_nodes!(input.into_children();
            [Date(date), String(module), StringOrAccount(options)..] => (date, module, options.collect()),
        );
        Ok(Directive::Custom {
            date: ret.0,
            type_name: ret.1,
            values: ret.2
        })
    }

    fn Include(input: Node) -> Result<Directive> {
        let ret: AvaroString = match_nodes!(input.into_children();
            [QuoteString(path)] => path,
        );
        Ok(Directive::Include { file: ret.to_string() })
    }

    fn Note(input: Node) -> Result<Directive> {
        let ret: (NaiveDate, Account, AvaroString) = match_nodes!(input.into_children();
            [Date(date), AccountName(a), String(path)] => (date, a, path),
        );
        Ok(Directive::Note {
            date: ret.0,
            account: ret.1,
            description: ret.2.to_string(),
        })
    }

    fn Pad(input: Node) -> Result<Directive> {
        let ret: (NaiveDate, Account, Account) = match_nodes!(input.into_children();
            [Date(date), AccountName(from), AccountName(to)] => (date, from, to),
        );
        Ok(Directive::Pad {
            date: ret.0,
            from: ret.1,
            to: ret.2,
        })
    }

    fn Event(input: Node) -> Result<Directive> {
        let ret: (NaiveDate, AvaroString, AvaroString) = match_nodes!(input.into_children();
            [Date(date), String(name), String(value)] => (date, name, value),
        );
        Ok(Directive::Event {
            date: ret.0,
            name: ret.1.to_string(),
            value: ret.2.to_string(),
        })
    }

    fn Balance(input: Node) -> Result<Directive> {
        let ret: (NaiveDate, Account, BigDecimal, String) = match_nodes!(input.into_children();
            [Date(date), AccountName(name), number(amount), CommodityName(commodity)] => (date, name, amount, commodity),
        );
        Ok(Directive::Balance {
            date: ret.0,
            account: ret.1,
            amount: (ret.2, ret.3),
        })
    }

    fn Document(input: Node) -> Result<Directive> {
        let ret: (NaiveDate, Account, AvaroString) = match_nodes!(input.into_children();
            [Date(date), AccountName(name), String(path)] => (date, name, path),
        );
        Ok(Directive::Document {
            date: ret.0,
            account: ret.1,
            path: ret.2.to_string(),
        })
    }

    fn Price(input: Node) -> Result<Directive> {
        let ret: (NaiveDate, String, BigDecimal, String) = match_nodes!(input.into_children();
            [Date(date), CommodityName(source), number(price), CommodityName(target)] => (date, source, price, target)
        );
        Ok(Directive::Price {
            date: ret.0,
            commodity: ret.1,
            amount: (ret.2, ret.3),
        })
    }


    fn Item(input: Node) -> Result<Directive> {
        let ret = match_nodes!(input.into_children();
            [Option(item)] => item,
            [Open(item)] => item,
            [Plugin(item)] => item,
            [Close(item)] => item,
            [Include(item)] => item,
            [Note(item)] => item,
            [Pad(item)] => item,
            [Event(item)] => item,
            [Document(item)] => item,
            [Balance(item)] => item,
            [Price(item)] => item,
            [Commodity(item)] => item,
            [Custom(item)] => item,
        );
        Ok(ret)
    }
    fn Entry(input: Node) -> Result<Vec<Directive>> {
        let ret = match_nodes!(input.into_children();
            [Item(items).., _] => items.collect(),
        );
        Ok(ret)
    }
}

pub fn parse_avaro(input_str: &str) -> Result<Vec<Directive>> {
    // Parse the input into `Nodes`
    let inputs = AvaroParser::parse(Rule::Entry, input_str)?;
    // There should be a single root node in the parsed tree
    let input = inputs.single()?;
    // Consume the `Node` recursively into the final value
    AvaroParser::Entry(input)
}

pub fn parse_account(input_str: &str) -> Result<Account> {
    // Parse the input into `Nodes`
    let inputs = AvaroParser::parse(Rule::AccountName, input_str)?;
    // There should be a single root node in the parsed tree
    let input = inputs.single()?;
    // Consume the `Node` recursively into the final value
    AvaroParser::AccountName(input)
}