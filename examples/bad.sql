-- This file has several lint issues

-- SQ001: SELECT *
select * from users;

-- SQ002: UPDATE without WHERE
UPDATE users SET name = 'hacked';

-- SQ002: DELETE without WHERE
delete from orders;

-- SQ003: Mixed keyword casing
SELECT id, name from users WHERE id = 1;

-- SQ005: Unused alias
SELECT id, name FROM users u;
