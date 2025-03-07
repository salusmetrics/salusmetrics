drop dictionary if exists SALUS_METRICS.regexp_os;

drop dictionary if exists SALUS_METRICS.regexp_browser;

drop dictionary if exists SALUS_METRICS.regexp_device;

create dictionary SALUS_METRICS.regexp_os (
    regex String,
    os_replacement String default 'Other',
    os_v1_replacement String default '0',
    os_v2_replacement String default '0',
    os_v3_replacement String default '0',
    os_v4_replacement String default '0'
) PRIMARY KEY (regex) SOURCE (
    YAMLRegExpTree (PATH './clickhouse_data/user_files/os.yaml')
) LIFETIME (0) LAYOUT (regexp_tree);

create dictionary SALUS_METRICS.regexp_browser (
    regex String,
    family_replacement String default 'Other',
    v1_replacement String default '0',
    v2_replacement String default '0'
) PRIMARY KEY (regex) SOURCE (
    YAMLRegExpTree (PATH './clickhouse_data/user_files/ua.yaml')
) LIFETIME (0) LAYOUT (regexp_tree);

create dictionary SALUS_METRICS.regexp_device (
    regex String,
    device_replacement String default 'Other',
    brand_replacement String,
    model_replacement String
) PRIMARY KEY (regex) SOURCE (
    YAMLRegExpTree (PATH './clickhouse_data/user_files/device.yaml')
) LIFETIME (0) LAYOUT (regexp_tree);

select
    -- user_agent,
    tupleElement (device_tuple, 1) as device_brand,
    tupleElement (device_tuple, 2) as device_model,
    tupleElement (os_tuple, 1) as os_name,
    concat (
        tupleElement (os_tuple, 2),
        '.',
        tupleElement (os_tuple, 3),
        '.',
        tupleElement (os_tuple, 4)
    ) as os_version,
    tupleElement (browser_tuple, 1) as browser_name,
    concat (
        tupleElement (browser_tuple, 2),
        '.',
        tupleElement (browser_tuple, 3)
    ) as browser_version
from
    (
        select
            user_agent,
            dictGet (
                'SALUS_METRICS.regexp_device',
                ('brand_replacement', 'device_replacement'),
                user_agent
            ) device_tuple,
            dictGet (
                'SALUS_METRICS.regexp_os',
                (
                    'os_replacement',
                    'os_v1_replacement',
                    'os_v2_replacement',
                    'os_v3_replacement'
                ),
                user_agent
            ) os_tuple,
            dictGet (
                'SALUS_METRICS.regexp_browser',
                (
                    'family_replacement',
                    'v1_replacement',
                    'v2_replacement'
                ),
                user_agent
            ) as browser_tuple
        from
            SALUS_METRICS.SESSION_EVENT
    )
order by
    user_agent
limit
    20;
