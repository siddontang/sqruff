SELECT
  id,
  name,
  email
FROM
  users
WHERE
  id = 1
  AND name = 'Alice'
ORDER BY
  id DESC
LIMIT 10;
