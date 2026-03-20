SELECT id, name, email
FROM users
WHERE (active = 1 OR role = 'admin')
    AND created_at > '2023-01-01'
    AND email LIKE '%@example.com'
    AND id IN (100, 200, 300)
    AND age BETWEEN 18 AND 65;
