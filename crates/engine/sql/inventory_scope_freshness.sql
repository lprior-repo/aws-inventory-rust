CREATE VIEW scope_freshness AS
WITH latest AS (
    SELECT sweep_id, scope_key, outcome, observed_at
    FROM (
        SELECT o.sweep_id, o.scope_key, o.outcome, o.observed_at,
               row_number() OVER (
                   PARTITION BY o.scope_key
                   ORDER BY o.observed_at DESC, o.sweep_id DESC
               ) AS selection_rank
        FROM scope_outcomes o
    ) WHERE selection_rank = 1
),
last_complete AS (
    SELECT scope_key, max(observed_at) AS last_complete_at
    FROM scope_outcomes
    WHERE outcome = 'complete'
    GROUP BY scope_key
),
f1 AS (
    SELECT s.*,
           (instr('0123456789ABCDEF', substr(h1, 1, 1)) - 1) * 4096
         + (instr('0123456789ABCDEF', substr(h1, 2, 1)) - 1) * 256
         + (instr('0123456789ABCDEF', substr(h1, 3, 1)) - 1) * 16
         + (instr('0123456789ABCDEF', substr(h1, 4, 1)) - 1) AS n1
    FROM (SELECT l.*, hex(substr(l.scope_key, 3, 2)) AS h1 FROM latest l) s
),
f2 AS (
    SELECT s.*,
           (instr('0123456789ABCDEF', substr(h2, 1, 1)) - 1) * 4096
         + (instr('0123456789ABCDEF', substr(h2, 2, 1)) - 1) * 256
         + (instr('0123456789ABCDEF', substr(h2, 3, 1)) - 1) * 16
         + (instr('0123456789ABCDEF', substr(h2, 4, 1)) - 1) AS n2
    FROM (SELECT f.*, hex(substr(f.scope_key, 5 + f.n1 + 2, 2)) AS h2 FROM f1 f) s
),
f3 AS (
    SELECT s.*,
           (instr('0123456789ABCDEF', substr(h3, 1, 1)) - 1) * 4096
         + (instr('0123456789ABCDEF', substr(h3, 2, 1)) - 1) * 256
         + (instr('0123456789ABCDEF', substr(h3, 3, 1)) - 1) * 16
         + (instr('0123456789ABCDEF', substr(h3, 4, 1)) - 1) AS n3
    FROM (SELECT f.*, hex(substr(f.scope_key, 9 + f.n1 + f.n2 + 2, 2)) AS h3 FROM f2 f) s
),
f4 AS (
    SELECT s.*,
           (instr('0123456789ABCDEF', substr(h4, 1, 1)) - 1) * 4096
         + (instr('0123456789ABCDEF', substr(h4, 2, 1)) - 1) * 256
         + (instr('0123456789ABCDEF', substr(h4, 3, 1)) - 1) * 16
         + (instr('0123456789ABCDEF', substr(h4, 4, 1)) - 1) AS n4
    FROM (SELECT f.*, hex(substr(f.scope_key, 13 + f.n1 + f.n2 + f.n3 + 2, 2)) AS h4 FROM f3 f) s
)
SELECT f.scope_key AS scope_key,
       CAST(substr(f.scope_key, 5, f.n1) AS TEXT) AS account,
       CAST(substr(f.scope_key, 9 + f.n1, f.n2) AS TEXT) AS region,
       CAST(substr(f.scope_key, 13 + f.n1 + f.n2, f.n3) AS TEXT) AS service,
       CAST(substr(f.scope_key, 17 + f.n1 + f.n2 + f.n3, f.n4) AS TEXT) AS operation,
       hex(substr(f.scope_key, 17 + f.n1 + f.n2 + f.n3 + f.n4, 1)) = '01' AS has_parent,
       f.sweep_id AS sweep_id,
       f.outcome AS outcome,
       f.observed_at AS observed_at,
       c.last_complete_at AS last_complete_at,
       CAST(unixepoch('subsec') * 1000 AS INTEGER) - f.observed_at AS age_ms,
       CAST(unixepoch('subsec') * 1000 AS INTEGER) - c.last_complete_at AS complete_age_ms
FROM f4 f LEFT JOIN last_complete c ON c.scope_key = f.scope_key;
