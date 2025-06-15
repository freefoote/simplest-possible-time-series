CREATE VIEW tsdata_code_binary_sizes AS
SELECT
id,
contents->>'commit' AS gitcommit,
(contents->'lines_of_code')::int AS lines_of_code,
(contents->'binary_size_bytes')::int AS binary_size_bytes,
(contents->'test_coverage_percent')::float AS test_coverage_percent,
(contents->>'date')::timestamptz AS relevant_time
FROM tsdata
WHERE series_name = 'code_binary_sizes'
ORDER BY relevant_time ASC;