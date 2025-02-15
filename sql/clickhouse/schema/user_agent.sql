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
    user_agent,
    concat (
        tupleElement (device, 1),
        ' ',
        tupleElement (device, 2)
    ) as device,
    concat (
        tupleElement (browser, 1),
        ' ',
        tupleElement (browser, 2),
        '.',
        tupleElement (browser, 3)
    ) as browser,
    concat (
        tupleElement (os, 1),
        ' ',
        tupleElement (os, 2),
        '.',
        tupleElement (os, 3),
        '.',
        tupleElement (os, 4)
    ) as os
from
    (
        select
            user_agent,
            dictGet (
                'SALUS_METRICS.regexp_os',
                (
                    'os_replacement',
                    'os_v1_replacement',
                    'os_v2_replacement',
                    'os_v3_replacement'
                ),
                user_agent
            ) os,
            dictGet (
                'SALUS_METRICS.regexp_browser',
                (
                    'family_replacement',
                    'v1_replacement',
                    'v2_replacement'
                ),
                user_agent
            ) as browser,
            dictGet (
                'SALUS_METRICS.regexp_device',
                ('brand_replacement', 'device_replacement'),
                user_agent
            ) device
        from
            SALUS_METRICS.SESSION_EVENT
    )
order by
    user_agent;
