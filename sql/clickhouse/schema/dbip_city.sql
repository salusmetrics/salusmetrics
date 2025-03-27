DROP TABLE IF EXISTS SALUS_METRICS.iso_3166_country_codes;

CREATE TABLE SALUS_METRICS.iso_3166_country_codes (
    `name` Nullable(String),
    `cca2` Nullable(String),
    `cca3` Nullable(String),
    `ccn` Nullable(String),
    `iso_3166_2` Nullable(String),
    `region` Nullable(String),
    `sub_region` Nullable(String),
    `intermediate_region` Nullable(String),
    `region_code` Nullable(String),
    `sub_region_code` Nullable(String),
    `intermediate_region_code` Nullable(String)
) engine = URL (
    'https://raw.githubusercontent.com/lukes/ISO-3166-Countries-with-Regional-Codes/master/all/all.csv',
    'CSV'
);


DROP TABLE IF EXISTS SALUS_METRICS.dbip_city_ipv4_url;

create table SALUS_METRICS.dbip_city_ipv4_url (
    ip_range_start IPv4,
    ip_range_end IPv4,
    country_code Nullable(String),
    state1 Nullable(String),
    state2 Nullable(String),
    city Nullable(String),
    postcode Nullable(String),
    latitude Float64,
    longitude Float64,
    timezone Nullable(String)
) engine = URL (
    'https://raw.githubusercontent.com/sapics/ip-location-db/master/dbip-city/dbip-city-ipv4.csv.gz',
    'CSV'
);

DROP DICTIONARY IF EXISTS SALUS_METRICS.dbip_city_ipv4_trie;
DROP TABLE IF EXISTS SALUS_METRICS.dbip_city_ipv4;

create table SALUS_METRICS.dbip_city_ipv4 (
   cidr String,
   latitude Float64,
   longitude Float64,
   country_code String,
   state  String,
   city String,
)
engine = MergeTree()
order by cidr;

insert into
    SALUS_METRICS.dbip_city_ipv4
with
    bitXor(ip_range_start, ip_range_end) as xor,
    if(xor != 0, ceil(log2(xor)), 0) as unmatched,
    32 - unmatched as cidr_suffix,
    toIPv4(bitAnd(bitNot(pow(2, unmatched) - 1), ip_range_start)::UInt64) as cidr_address
select
    concat(toString(cidr_address),'/',toString(cidr_suffix)) as cidr,
    latitude,
    longitude,
    coalesce(SALUS_METRICS.iso_3166_country_codes.cca3, '') as country_code,
    coalesce(state1, '') as state,
    coalesce(city, '') as city
from
    SALUS_METRICS.dbip_city_ipv4_url
left outer join SALUS_METRICS.iso_3166_country_codes
    on (SALUS_METRICS.dbip_city_ipv4_url.country_code = SALUS_METRICS.iso_3166_country_codes.cca2);

create dictionary dbip_city_ipv4_trie (
    cidr String,
    latitude Float64,
    longitude Float64,
    country_code String,
    state String,
    city String
)
primary key cidr
source(clickhouse(table ‘dbip_city_ipv4’))
layout(ip_trie)
lifetime(3600);
