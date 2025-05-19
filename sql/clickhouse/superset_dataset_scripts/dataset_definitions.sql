
SELECT
    country_code,
    COUNT() as page_views,
    uniqCombined(visitor) as visitors
FROM SALUS_METRICS.SECTION_COMBINED
WHERE (
  api_key in {{filter_values('api_key')|where_in}}
  and site in {{filter_values('site')|where_in}}
  {% if filter_values('browser')|length
    or filter_values('city')|length
    or filter_values('os')|length
    or filter_values('path')|length %}
    {% if from_dttm is not none and to_dttm is not none %}
        AND
        ts >= toDateTime('{{ from_dttm }}') AND
        ts < toDateTime('{{ to_dttm }}')
    {% endif %}
  {% else %}
    {% if from_dttm is not none and to_dttm is not none %}
        {% if date_range_days(from_dttm, to_dttm) > 10 %}
            AND
            toStartOfDay(ts) >= toDateTime('{{ from_dttm }}') AND
            toStartOfDay(ts) < toDateTime('{{ to_dttm }}')
        {% else %}
            AND
            ts >= toDateTime('{{ from_dttm }}') AND
            ts < toDateTime('{{ to_dttm }}')
        {% endif %}
    {% endif %}
  {% endif %}
  {% if filter_values('browser')|length %}
	  and browser in {{filter_values('browser')|where_in}}
  {% endif %}
  {% if filter_values('city')|length %}
    and city in {{filter_values('city')|where_in}}
  {% endif %}
  {% if filter_values('os')|length %}
    and os in {{filter_values('os')|where_in}}
  {% endif %}
  {% if filter_values('path')|length %}
    and path in {{filter_values('path')|where_in}}
  {% endif %}
)
GROUP BY country_code


-----------------------------------------
-----------------------------------------

SELECT
    {% if date_range_days(from_dttm, to_dttm) > 10 %}
      toStartOfDay(`ts`) as time,
    {% elif date_range_days(from_dttm, to_dttm) > 1 %}
      toStartOfHour(`ts`) as time,
    {% else %}
      toStartOfFiveMinutes(`ts`) as time,
    {% endif %}
    COUNT() as page_views,
    uniqCombined(visitor) as visitors
FROM SALUS_METRICS.SECTION_COMBINED
WHERE (
    api_key in {{filter_values('api_key')|where_in}}
    and site in {{filter_values('site')|where_in}}
  {% if from_dttm is not none and to_dttm is not none %}
    and time >= toDateTime('{{ from_dttm }}')
    and time < toDateTime('{{ to_dttm }}')
  {% endif %}
  {% if filter_values('browser')|length %}
	  and browser in {{filter_values('browser')|where_in}}
  {% endif %}
  {% if filter_values('city')|length %}
	  and city in {{filter_values('city')|where_in}}
  {% endif %}
  {% if filter_values('country_code')|length %}
	  and country_code in {{filter_values('country_code')|where_in}}
  {% endif %}
  {% if filter_values('os')|length %}
	  and os in {{filter_values('os')|where_in}}
  {% endif %}
  {% if filter_values('path')|length %}
	  and path in {{filter_values('path')|where_in}}
  {% endif %}
)
GROUP BY time


-----------------------------------------
-----------------------------------------
SELECT
    uniqCombined(visitor) as visitors
FROM SALUS_METRICS.SECTION_COMBINED
WHERE (
    api_key in {{filter_values('api_key')|where_in}}
    and site in {{filter_values('site')|where_in}}
  {% if from_dttm is not none and to_dttm is not none %}
    {% if date_range_days(from_dttm, to_dttm) > 10 %}
        and toStartOfDay(`ts`) >= toDateTime('{{ from_dttm }}')
        and toStartOfDay(`ts`) < toDateTime('{{ to_dttm }}')
    {% else %}
        and ts >= toDateTime('{{ from_dttm }}')
        and ts < toDateTime('{{ to_dttm }}')
    {% endif %}
  {% endif %}
  {% if filter_values('browser')|length %}
	  and browser in {{filter_values('browser')|where_in}}
  {% endif %}
  {% if filter_values('city')|length %}
	  and city in {{filter_values('city')|where_in}}
  {% endif %}
  {% if filter_values('country_code')|length %}
	  and country_code in {{filter_values('country_code')|where_in}}
  {% endif %}
  {% if filter_values('os')|length %}
	  and os in {{filter_values('os')|where_in}}
  {% endif %}
  {% if filter_values('path')|length %}
	  and path in {{filter_values('path')|where_in}}
  {% endif %}
)

-----------------------------------------
-----------------------------------------

SELECT
    browser,
    COUNT() as page_views
FROM SALUS_METRICS.SECTION_COMBINED
WHERE (
  api_key in {{filter_values('api_key')|where_in}}
  and site in {{filter_values('site')|where_in}}
  {% if filter_values('city')|length
    or filter_values('country_code')|length
    or filter_values('os')|length
    or filter_values('path')|length %}
    {% if from_dttm is not none and to_dttm is not none %}
        AND
        ts >= toDateTime('{{ from_dttm }}') AND
        ts < toDateTime('{{ to_dttm }}')
    {% endif %}
  {% else %}
    {% if from_dttm is not none and to_dttm is not none %}
        {% if date_range_days(from_dttm, to_dttm) > 10 %}
            AND
            toStartOfDay(ts) >= toDateTime('{{ from_dttm }}') AND
            toStartOfDay(ts) < toDateTime('{{ to_dttm }}')
        {% else %}
            AND
            ts >= toDateTime('{{ from_dttm }}') AND
            ts < toDateTime('{{ to_dttm }}')
        {% endif %}
    {% endif %}
  {% endif %}
  {% if filter_values('city')|length %}
    and city in {{filter_values('city')|where_in}}
  {% endif %}
  {% if filter_values('country_code')|length %}
    and country_code in {{filter_values('country_code')|where_in}}
  {% endif %}
  {% if filter_values('os')|length %}
    and os in {{filter_values('os')|where_in}}
  {% endif %}
  {% if filter_values('path')|length %}
    and path in {{filter_values('path')|where_in}}
  {% endif %}
)
GROUP BY browser

-----------------------------------------
-----------------------------------------

SELECT
    os,
    COUNT() as page_views
FROM SALUS_METRICS.SECTION_COMBINED
WHERE (
  api_key in {{filter_values('api_key')|where_in}}
  and site in {{filter_values('site')|where_in}}
  {% if filter_values('browser')|length
    or filter_values('city')|length
    or filter_values('country_code')|length
    or filter_values('path')|length %}
    {% if from_dttm is not none and to_dttm is not none %}
        AND
        ts >= toDateTime('{{ from_dttm }}') AND
        ts < toDateTime('{{ to_dttm }}')
    {% endif %}
  {% else %}
    {% if from_dttm is not none and to_dttm is not none %}
        {% if date_range_days(from_dttm, to_dttm) > 10 %}
            AND
            toStartOfDay(ts) >= toDateTime('{{ from_dttm }}') AND
            toStartOfDay(ts) < toDateTime('{{ to_dttm }}')
        {% else %}
            AND
            ts >= toDateTime('{{ from_dttm }}') AND
            ts < toDateTime('{{ to_dttm }}')
        {% endif %}
    {% endif %}
  {% endif %}
  {% if filter_values('browser')|length %}
    and browser in {{filter_values('browser')|where_in}}
  {% endif %}
  {% if filter_values('city')|length %}
    and city in {{filter_values('city')|where_in}}
  {% endif %}
  {% if filter_values('country_code')|length %}
    and country_code in {{filter_values('country_code')|where_in}}
  {% endif %}
  {% if filter_values('path')|length %}
    and path in {{filter_values('path')|where_in}}
  {% endif %}
)
GROUP BY os

-----------------------------------------
-----------------------------------------

SELECT
    path,
    COUNT() as page_views
FROM SALUS_METRICS.SECTION_COMBINED
WHERE (
  api_key in {{filter_values('api_key')|where_in}}
  and site in {{filter_values('site')|where_in}}
  {% if filter_values('browser')|length
    or filter_values('city')|length
    or filter_values('country_code')|length
    or filter_values('path')|length %}
    {% if from_dttm is not none and to_dttm is not none %}
        AND
        ts >= toDateTime('{{ from_dttm }}') AND
        ts < toDateTime('{{ to_dttm }}')
    {% endif %}
  {% else %}
    {% if from_dttm is not none and to_dttm is not none %}
        {% if date_range_days(from_dttm, to_dttm) > 10 %}
            AND
            toStartOfDay(ts) >= toDateTime('{{ from_dttm }}') AND
            toStartOfDay(ts) < toDateTime('{{ to_dttm }}')
        {% else %}
            AND
            ts >= toDateTime('{{ from_dttm }}') AND
            ts < toDateTime('{{ to_dttm }}')
        {% endif %}
    {% endif %}
  {% endif %}
  {% if filter_values('browser')|length %}
    and browser in {{filter_values('browser')|where_in}}
  {% endif %}
  {% if filter_values('city')|length %}
    and city in {{filter_values('city')|where_in}}
  {% endif %}
  {% if filter_values('country_code')|length %}
    and country_code in {{filter_values('country_code')|where_in}}
  {% endif %}
  {% if filter_values('os')|length %}
    and os in {{filter_values('os')|where_in}}
  {% endif %}
)
GROUP BY path


-----------------------------------------
-----------------------------------------

SELECT
    city,
    COUNT() as page_views
FROM SALUS_METRICS.SECTION_COMBINED
WHERE (
  api_key in {{filter_values('api_key')|where_in}}
  and site in {{filter_values('site')|where_in}}
  {% if filter_values('browser')|length
    or filter_values('city')|length
    or filter_values('country_code')|length
    or filter_values('path')|length %}
    {% if from_dttm is not none and to_dttm is not none %}
        AND
        ts >= toDateTime('{{ from_dttm }}') AND
        ts < toDateTime('{{ to_dttm }}')
    {% endif %}
  {% else %}
    {% if from_dttm is not none and to_dttm is not none %}
        {% if date_range_days(from_dttm, to_dttm) > 10 %}
            AND
            toStartOfDay(ts) >= toDateTime('{{ from_dttm }}') AND
            toStartOfDay(ts) < toDateTime('{{ to_dttm }}')
        {% else %}
            AND
            ts >= toDateTime('{{ from_dttm }}') AND
            ts < toDateTime('{{ to_dttm }}')
        {% endif %}
    {% endif %}
  {% endif %}
  {% if filter_values('browser')|length %}
    and browser in {{filter_values('browser')|where_in}}
  {% endif %}
  {% if filter_values('country_code')|length %}
    and country_code in {{filter_values('country_code')|where_in}}
  {% endif %}
  {% if filter_values('os')|length %}
    and os in {{filter_values('os')|where_in}}
  {% endif %}
  {% if filter_values('path')|length %}
    and path in {{filter_values('path')|where_in}}
  {% endif %}
)
GROUP BY city


-----------------------------------------
-----------------------------------------


SELECT
    {% if date_range_days(from_dttm, to_dttm) > 30 %}
      toStartOfDay(`ts`) as ts,
    {% elif date_range_days(from_dttm, to_dttm) > 1 %}
      toStartOfHour(`ts`) as ts,
    {% else %}
      toStartOfFiveMinutes(`ts`) as ts,
    {% endif %}
    sumMerge(visitors) as visitors
FROM VISITOR_TIMESERIES
WHERE (
    api_key in {{filter_values('api_key')|where_in}}
    and site in {{filter_values('site')|where_in}}
    and ts >= toDateTime('{{ from_dttm }}')
    and ts < toDateTime('{{ to_dttm }}')
)
GROUP BY api_key, site, ts
