-- This is a line comment
SELECT id, /* inline block comment */ name
FROM users -- trailing comment
WHERE active = 1;
