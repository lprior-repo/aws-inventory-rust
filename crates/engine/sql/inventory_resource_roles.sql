CREATE VIEW resource_roles(account,region,kind,id,role,tag_key,value) AS
WITH candidates AS (
  SELECT t.account,t.region,t.kind,t.id,m.role,m.position,t.tag_key,t.tag_value
  FROM tag_mapping m
  JOIN tags t ON t.tag_key=m.tag_key
  WHERE TRIM(t.tag_value)<>''
),
ranked AS (
  SELECT account,region,kind,id,role,tag_key,tag_value,
    ROW_NUMBER() OVER (PARTITION BY account,region,kind,id,role ORDER BY position) AS pick
  FROM candidates
),
roles(role) AS (
  VALUES('owner'),('team'),('application')
)
SELECT r.account,r.region,r.kind,r.id,roles.role,c.tag_key,c.tag_value
FROM resources r
CROSS JOIN roles
LEFT JOIN ranked c
  ON c.account=r.account AND c.region=r.region AND c.kind=r.kind AND c.id=r.id
  AND c.role=roles.role AND c.pick=1;
