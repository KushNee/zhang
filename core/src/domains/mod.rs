use crate::database::type_ext::big_decimal::ZhangBigDecimal;
use crate::domains::schemas::{
    AccountBalanceDomain, AccountDailyBalanceDomain, AccountDomain, AccountJournalDomain, CommodityDomain, ErrorDomain, ErrorType, MetaDomain, MetaType,
    OptionDomain, PriceDomain, TransactionInfoDomain,
};
use crate::store::Store;
use crate::ZhangResult;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use itertools::Itertools;
use serde::Deserialize;
use sqlx::pool::PoolConnection;
use sqlx::{Acquire, FromRow, Sqlite};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use uuid::Uuid;
use zhang_ast::{Meta, SpanInfo};

pub mod schemas;

#[derive(FromRow)]
struct ValueRow {
    value: String,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct AccountAmount {
    pub number: ZhangBigDecimal,
    pub commodity: String,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct LotRow {
    pub amount: f64,
    pub price_amount: Option<f64>,
    pub price_commodity: Option<String>,
}

#[derive(FromRow)]
pub struct StaticRow {
    pub date: NaiveDate,
    pub account_type: String,
    pub amount: ZhangBigDecimal,
    pub commodity: String,
}

pub struct Operations {
    #[cfg(feature = "sqlite")]
    pub(crate) pool: PoolConnection<Sqlite>,
    pub timezone: Tz,
    pub store: Arc<RwLock<Store>>,
}

impl Operations {
    pub fn read(&self) -> RwLockReadGuard<Store> {
        self.store.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<Store> {
        self.store.write().unwrap()
    }
}

impl Operations {
    pub(crate) async fn insert_or_update_account(
        &mut self, datetime: DateTime<Tz>, account_type: String, account_name: &str, status: &str, alias: Option<&str>,
    ) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;
        sqlx::query(r#"INSERT OR REPLACE INTO accounts(date, type, name, status, alias) VALUES ($1, $2, $3, $4, $5);"#)
            .bind(datetime)
            .bind(account_type)
            .bind(account_name)
            .bind(status)
            .bind(alias)
            .execute(conn)
            .await?;
        Ok(())
    }
    pub(crate) async fn insert_transaction(
        &mut self, id: &String, datetime: DateTime<Tz>, flag: String, payee: Option<&str>, narration: Option<&str>, filename: Option<&str>, span_start: i64,
        span_end: i64,
    ) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;

        sqlx::query(
            r#"INSERT INTO transactions (id, datetime, type, payee, narration, source_file, span_start, span_end)VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(id)
        .bind(datetime)
        .bind(flag)
        .bind(payee)
        .bind(narration)
        .bind(filename)
        .bind(span_start)
        .bind(span_end)
        .execute(conn)
        .await?;
        Ok(())
    }

    pub(crate) async fn insert_transaction_posting(
        &mut self, id: &String, account_name: &str, unit_number: Option<String>, unit_commodity: Option<&String>, cost_number: Option<String>,
        cost_commodity: Option<&String>, inferred_amount_number: String, inferred_amount_commodity: &String, previous_number: &ZhangBigDecimal,
        previous_commodity: &String, after_number: String, after_commodity: &String,
    ) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;

        sqlx::query(
            r#"INSERT INTO transaction_postings
                               (trx_id, account, unit_number, unit_commodity, cost_number, cost_commodity, inferred_unit_number, inferred_unit_commodity,
                                account_before_number, account_before_commodity, account_after_number, account_after_commodity
                               )
                               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
        )
        .bind(id)
        .bind(account_name)
        .bind(unit_number)
        .bind(unit_commodity)
        .bind(cost_number)
        .bind(cost_commodity)
        .bind(inferred_amount_number)
        .bind(inferred_amount_commodity)
        .bind(previous_number)
        .bind(previous_commodity)
        .bind(after_number)
        .bind(after_commodity)
        .execute(conn)
        .await?;
        Ok(())
    }

    pub(crate) async fn insert_transaction_tag(&mut self, id: &String, tag: &String) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;

        sqlx::query(r#"INSERT INTO transaction_tags (trx_id, tag)VALUES ($1, $2)"#)
            .bind(id)
            .bind(tag)
            .execute(conn)
            .await?;
        Ok(())
    }
    pub(crate) async fn insert_transaction_link(&mut self, id: &String, link: &String) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;

        sqlx::query(r#"INSERT INTO transaction_tags (trx_id, tag)VALUES ($1, $2)"#)
            .bind(id)
            .bind(link)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub(crate) async fn insert_trx_document(
        &mut self, datetime: DateTime<Tz>, filename: Option<&str>, path: &str, extension: Option<&str>, trx_id: &str,
    ) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;

        sqlx::query(r#"INSERT INTO documents (datetime, filename, path, extension, trx_id) VALUES ($1, $2, $3, $4, $5);"#)
            .bind(datetime)
            .bind(filename)
            .bind(path)
            .bind(extension)
            .bind(trx_id)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub(crate) async fn insert_account_document(
        &mut self, datetime: DateTime<Tz>, filename: Option<&str>, path: &str, extension: Option<&str>, account_name: &str,
    ) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;

        sqlx::query(r#"INSERT INTO documents (datetime, filename, path, extension, account) VALUES ($1, $2, $3, $4, $5);"#)
            .bind(datetime)
            .bind(filename)
            .bind(path)
            .bind(extension)
            .bind(account_name)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub(crate) async fn insert_price(&mut self, datetime: DateTime<Tz>, commodity: &str, amount: &BigDecimal, target_commodity: &str) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;

        sqlx::query(r#"INSERT INTO prices (datetime, commodity, amount, target_commodity)VALUES ($1, $2, $3, $4)"#)
            .bind(datetime)
            .bind(commodity)
            .bind(amount.to_string())
            .bind(target_commodity)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub(crate) async fn account_target_day_balance(
        &mut self, account_name: &str, datetime: DateTime<Tz>, currency: &str,
    ) -> ZhangResult<Option<AccountAmount>> {
        let conn = self.pool.acquire().await?;

        let option: Option<AccountAmount> = sqlx::query_as(
            r#"select account_after_number as number, account_after_commodity as commodity from transaction_postings
                                join transactions on transactions.id = transaction_postings.trx_id
                                where account = $1 and "datetime" <=  $2 and account_after_commodity = $3
                                order by "datetime" desc, transactions.sequence desc limit 1"#,
        )
        .bind(account_name)
        .bind(datetime)
        .bind(currency)
        .fetch_optional(conn)
        .await?;
        Ok(option)
    }

    pub(crate) async fn account_lot(
        &mut self, account_name: &str, currency: &str, price_amount: &BigDecimal, price_commodity: &str,
    ) -> ZhangResult<Option<LotRow>> {
        let conn = self.pool.acquire().await?;

        let lot: Option<LotRow> = sqlx::query_as(
            r#"
            select amount, price_amount, price_commodity
            from commodity_lots
            where account = $1 and commodity = $2 and price_amount = $3 and price_commodity = $4"#,
        )
        .bind(account_name)
        .bind(currency)
        .bind(price_amount.to_string())
        .bind(price_commodity)
        .fetch_optional(conn)
        .await?;
        Ok(lot)
    }

    pub(crate) async fn account_lot_fifo(&mut self, account_name: &str, currency: &str, price_commodity: &str) -> ZhangResult<Option<LotRow>> {
        let conn = self.pool.acquire().await?;

        let lot: Option<LotRow> = sqlx::query_as(
            r#"
                select amount, price_amount, price_commodity
                from commodity_lots
                where account = $1 and commodity = $2
                  and (price_commodity = $3 or price_commodity is null)
                  and ((amount != 0 and price_amount is not null) or price_amount is null)
                order by datetime desc
            "#,
        )
        .bind(account_name)
        .bind(currency)
        .bind(price_commodity)
        .fetch_optional(conn)
        .await?;
        Ok(lot)
    }
    pub(crate) async fn update_account_lot(
        &mut self, account_name: &str, currency: &str, price_amount: &BigDecimal, price_commodity: &str, amount: &BigDecimal,
    ) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;

        sqlx::query(
            r#"update commodity_lots
                        set amount = $1
                        where account = $2 and commodity = $3  and price_amount = $4 and price_commodity = $5"#,
        )
        .bind(amount.to_string())
        .bind(account_name)
        .bind(currency)
        .bind(price_amount.to_string())
        .bind(price_commodity)
        .execute(conn)
        .await?;
        Ok(())
    }

    pub(crate) async fn update_account_default_lot(&mut self, account_name: &str, currency: &str, amount: &BigDecimal) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;

        sqlx::query(
            r#"update commodity_lots
                        set amount = $1
                        where account = $2 and commodity = $3  and price_amount is NULL and price_commodity is NULL"#,
        )
        .bind(amount.to_string())
        .bind(account_name)
        .bind(currency)
        .execute(conn)
        .await?;
        Ok(())
    }

    pub(crate) async fn insert_account_lot(
        &mut self, account_name: &str, currency: &str, price_amount: Option<&BigDecimal>, price_commodity: Option<&str>, amount: &BigDecimal,
    ) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;

        sqlx::query(
            r#"INSERT INTO commodity_lots (account, commodity, datetime, amount, price_amount, price_commodity)
                                    VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(account_name)
        .bind(currency)
        .bind(None::<NaiveDateTime>)
        .bind(amount.to_string())
        .bind(price_amount.map(|it| it.to_string()))
        .bind(price_commodity)
        .execute(conn)
        .await?;
        Ok(())
    }
}

impl Operations {
    pub async fn options(&mut self) -> ZhangResult<Vec<OptionDomain>> {
        let store = self.read();

        Ok(store.options.clone().into_iter().map(|(key, value)| OptionDomain { key, value }).collect_vec())
    }

    pub async fn option(&mut self, key: impl AsRef<str>) -> ZhangResult<Option<OptionDomain>> {
        let store = self.read();

        Ok(store.options.get(key.as_ref()).map(|value| OptionDomain {
            key: key.as_ref().to_string(),
            value: value.to_owned(),
        }))
    }

    pub async fn accounts_latest_balance(&mut self) -> ZhangResult<Vec<AccountDailyBalanceDomain>> {
        let conn = self.pool.acquire().await?;
        Ok(sqlx::query_as::<_, AccountDailyBalanceDomain>(
            r#"
                SELECT
                    date(datetime) AS date,
                    account,
                    balance_number,
                    balance_commodity
                FROM
                    account_daily_balance
                GROUP BY
                    account
                HAVING
                    max(datetime)
            "#,
        )
        .fetch_all(conn)
        .await?)
    }

    pub async fn get_price(&mut self, date: NaiveDateTime, from: impl AsRef<str>, to: impl AsRef<str>) -> ZhangResult<Option<PriceDomain>> {
        let datetime = self.timezone.from_local_datetime(&date).unwrap();
        let conn = self.pool.acquire().await?;
        Ok(sqlx::query_as::<_, PriceDomain>(
            "select datetime, commodity, amount, target_commodity from prices where datetime <= $1 and commodity = $2 and target_commodity = $3",
        )
        .bind(datetime)
        .bind(from.as_ref())
        .bind(to.as_ref())
        .fetch_optional(conn)
        .await?)
    }

    pub async fn metas(&mut self, type_: MetaType, type_identifier: impl AsRef<str>) -> ZhangResult<Vec<MetaDomain>> {
        let conn = self.pool.acquire().await?;

        let rows = sqlx::query_as::<_, MetaDomain>(
            r#"
            select type as meta_type, type_identifier, key, value from metas where type = $1 and type_identifier = $2
        "#,
        )
        .bind(type_.as_ref())
        .bind(type_identifier.as_ref())
        .fetch_all(conn)
        .await?;
        Ok(rows)
    }

    pub async fn trx_tags(&mut self, trx_id: impl AsRef<str>) -> ZhangResult<Vec<String>> {
        let conn = self.pool.acquire().await?;

        let rows = sqlx::query_as::<_, ValueRow>(
            r#"
        select tag as value from transaction_tags where trx_id = $1
        "#,
        )
        .bind(trx_id.as_ref())
        .fetch_all(conn)
        .await?;
        Ok(rows.into_iter().map(|it| it.value).collect_vec())
    }

    pub async fn trx_links(&mut self, trx_id: impl AsRef<str>) -> ZhangResult<Vec<String>> {
        let conn = self.pool.acquire().await?;

        let rows = sqlx::query_as::<_, ValueRow>(
            r#"
        select link as value from transaction_links where trx_id = $1
        "#,
        )
        .bind(trx_id.as_ref())
        .fetch_all(conn)
        .await?;
        Ok(rows.into_iter().map(|it| it.value).collect_vec())
    }

    pub async fn commodity(&mut self, name: &str) -> ZhangResult<Option<CommodityDomain>> {
        let conn = self.pool.acquire().await?;

        let option = sqlx::query_as::<_, CommodityDomain>(
            r#"
                select * from commodities where name = $1
                "#,
        )
        .bind(name)
        .fetch_optional(conn)
        .await?;
        Ok(option)
    }
    pub async fn exist_commodity(&mut self, name: &str) -> ZhangResult<bool> {
        let conn = self.pool.acquire().await?;

        Ok(sqlx::query("select 1 from commodities where name = $1")
            .bind(name)
            .fetch_optional(conn)
            .await?
            .is_some())
    }

    pub async fn exist_account(&mut self, name: &str) -> ZhangResult<bool> {
        let conn = self.pool.acquire().await?;

        Ok(sqlx::query("select 1 from accounts where name = $1")
            .bind(name)
            .fetch_optional(conn)
            .await?
            .is_some())
    }

    pub async fn transaction_counts(&mut self) -> ZhangResult<i64> {
        let conn = self.pool.acquire().await?;
        Ok(sqlx::query_as::<_, (i64,)>(r#"select count(1) from transactions"#).fetch_one(conn).await?.0)
    }

    pub async fn transaction_span(&mut self, id: &str) -> ZhangResult<TransactionInfoDomain> {
        let conn = self.pool.acquire().await?;
        Ok(
            sqlx::query_as::<_, TransactionInfoDomain>(r#"select id, source_file, span_start, span_end from transactions where id = $1"#)
                .bind(id)
                .fetch_one(conn)
                .await?,
        )
    }

    pub async fn account_balances(&mut self) -> ZhangResult<Vec<AccountBalanceDomain>> {
        let conn = self.pool.acquire().await?;
        Ok(sqlx::query_as::<_, AccountBalanceDomain>(
            r#"
                        select datetime, account, account_status, balance_number, balance_commodity
                        from account_balance
            "#,
        )
        .fetch_all(conn)
        .await?)
    }

    pub async fn single_account_balances(&mut self, account_name: &str) -> ZhangResult<Vec<AccountBalanceDomain>> {
        let conn = self.pool.acquire().await?;
        Ok(sqlx::query_as::<_, AccountBalanceDomain>(
            r#"
                select datetime, account, account_status, balance_number, balance_commodity
                from account_balance
                where account = $1
            "#,
        )
        .bind(account_name)
        .fetch_all(conn)
        .await?)
    }

    pub async fn account_journals(&mut self, account: &str) -> ZhangResult<Vec<AccountJournalDomain>> {
        let conn = self.pool.acquire().await?;
        Ok(sqlx::query_as::<_, AccountJournalDomain>(
            r#"
                    select datetime,
                           trx_id,
                           account,
                           payee,
                           narration,
                           inferred_unit_number,
                           inferred_unit_commodity,
                           account_after_number,
                           account_after_commodity
                    from transaction_postings
                             join transactions on transactions.id = transaction_postings.trx_id
                    where account = $1
                    order by datetime desc, transactions.sequence desc
            "#,
        )
        .bind(account)
        .fetch_all(conn)
        .await?)
    }
    pub async fn account_dated_journals(&mut self, account_type: &str, from: NaiveDateTime, to: NaiveDateTime) -> ZhangResult<Vec<AccountJournalDomain>> {
        let conn = self.pool.acquire().await?;
        Ok(sqlx::query_as::<_, AccountJournalDomain>(
            r#"
                select datetime,
                       trx_id,
                       account,
                       payee,
                       narration,
                       inferred_unit_number,
                       inferred_unit_commodity,
                       account_after_number,
                       account_after_commodity
                from transaction_postings
                         join transactions on transactions.id = transaction_postings.trx_id
                         join accounts on accounts.name = transaction_postings.account
                where datetime >= $1
                  and datetime <= $2
                  and accounts.type = $3
            "#,
        )
        .bind(from)
        .bind(to)
        .bind(account_type)
        .fetch_all(conn)
        .await?)
    }

    pub async fn errors(&mut self) -> ZhangResult<Vec<ErrorDomain>> {
        let conn = self.pool.acquire().await?;

        #[derive(FromRow)]
        struct ErrorRow {
            pub id: String,
            pub filename: Option<String>,
            pub span_start: Option<i64>,
            pub span_end: Option<i64>,
            pub content: String,
            pub error_type: ErrorType,
            pub metas: String,
        }

        let rows = sqlx::query_as::<_, ErrorRow>(
            r#"
            select
                id, filename, span_start, span_end, content, content, error_type, metas
            from errors
        "#,
        )
        .fetch_all(conn)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let span = match (row.span_start, row.span_end) {
                    (Some(start), Some(end)) => Some(SpanInfo {
                        start: start as usize,
                        end: end as usize,
                        content: row.content,
                        filename: row.filename.map(PathBuf::from),
                    }),
                    _ => None,
                };
                ErrorDomain {
                    id: row.id,
                    span,
                    error_type: row.error_type,
                    metas: serde_json::from_str(&row.metas).unwrap(),
                }
            })
            .collect_vec())
    }

    pub async fn account(&mut self, account_name: &str) -> ZhangResult<Option<AccountDomain>> {
        let conn = self.pool.acquire().await?;
        Ok(
            sqlx::query_as::<_, AccountDomain>(r#"select date, type, name, status, alias from accounts where name = $1"#)
                .bind(account_name)
                .fetch_optional(conn)
                .await?,
        )
    }
    pub async fn all_open_accounts(&mut self) -> ZhangResult<Vec<AccountDomain>> {
        let conn = self.pool.acquire().await?;
        Ok(
            sqlx::query_as::<_, AccountDomain>(r#"select date, type, name, status, alias from accounts WHERE status = 'Open'"#)
                .fetch_all(conn)
                .await?,
        )
    }
    pub async fn all_accounts(&mut self) -> ZhangResult<Vec<String>> {
        let conn = self.pool.acquire().await?;
        let accounts = sqlx::query_as::<_, ValueRow>("select name as value from accounts")
            .fetch_all(conn)
            .await?
            .into_iter()
            .map(|it| it.value)
            .collect_vec();
        Ok(accounts)
    }

    pub async fn all_payees(&mut self) -> ZhangResult<Vec<String>> {
        let conn = self.pool.acquire().await?;

        #[derive(FromRow)]
        struct PayeeRow {
            payee: String,
        }
        let payees = sqlx::query_as::<_, PayeeRow>(
            r#"
        select distinct payee from transactions
        "#,
        )
        .fetch_all(conn)
        .await?;
        Ok(payees.into_iter().map(|it| it.payee).filter(|it| !it.is_empty()).collect_vec())
    }

    pub async fn static_duration(&mut self, from: NaiveDateTime, to: NaiveDateTime) -> ZhangResult<Vec<StaticRow>> {
        let conn = self.pool.acquire().await?;
        let rows = sqlx::query_as::<_, StaticRow>(
            r#"
        SELECT
            date(datetime) AS date,
            accounts.type AS account_type,
            sum(inferred_unit_number) AS amount,
            inferred_unit_commodity AS commodity
        FROM
            transaction_postings
            JOIN transactions ON transactions.id = transaction_postings.trx_id
            JOIN accounts ON accounts.name = transaction_postings.account
            where transactions.datetime >= $1 and transactions.datetime <= $2
        GROUP BY
            date(datetime),
            accounts.type,
            inferred_unit_commodity
    "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(conn)
        .await?;

        Ok(rows)
    }
}

// for insert and new operations
impl Operations {
    pub async fn new_error(&mut self, error_type: ErrorType, span: &SpanInfo, metas: HashMap<String, String>) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO
                errors(id, filename, span_start, span_end, content, error_type, metas)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7);
        "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(span.filename.as_ref().and_then(|it| it.to_str()))
        .bind(span.start as i64)
        .bind(span.end as i64)
        .bind(&span.content)
        .bind(error_type)
        .bind(serde_json::to_string(&metas).unwrap())
        .execute(conn)
        .await?;
        Ok(())
    }

    pub async fn insert_or_update_options(&mut self, key: &str, value: &str) -> ZhangResult<()> {
        let mut store = self.write();

        store.options.insert(key.to_owned(), value.to_owned());
        Ok(())
    }

    pub async fn insert_meta(&mut self, type_: MetaType, type_identifier: impl AsRef<str>, meta: Meta) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;
        for (meta_key, meta_value) in meta.get_flatten() {
            sqlx::query(r#"INSERT OR REPLACE INTO metas VALUES ($1, $2, $3, $4);"#)
                .bind(type_.as_ref())
                .bind(type_identifier.as_ref())
                .bind(meta_key)
                .bind(meta_value.as_str())
                .execute(&mut *conn)
                .await?;
        }
        Ok(())
    }

    pub async fn close_account(&mut self, account_name: &str) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;
        sqlx::query(r#"update accounts set status = 'Close' where name = $1"#)
            .bind(account_name)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn insert_commodity(
        &mut self, name: &String, precision: Option<i32>, prefix: Option<String>, suffix: Option<String>, rounding: Option<String>,
    ) -> ZhangResult<()> {
        let conn = self.pool.acquire().await?;
        sqlx::query(
            r#"INSERT OR REPLACE INTO commodities (name, precision, prefix, suffix, rounding)
                        VALUES ($1, $2, $3, $4, $5);"#,
        )
        .bind(name)
        .bind(precision)
        .bind(prefix)
        .bind(suffix)
        .bind(rounding)
        .execute(conn)
        .await?;
        Ok(())
    }
}
