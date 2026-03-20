-- A well-written SQL file
SELECT
  id,
  name,
  email
FROM users
WHERE id = 1;

INSERT INTO orders (user_id, total)
VALUES (1, 99.99);

UPDATE orders
SET total = 149.99
WHERE id = 42;

DELETE FROM sessions
WHERE expires_at < '2024-01-01';

CREATE TABLE products (
  id INT PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  price DECIMAL(10, 2) DEFAULT 0.00,
  created_at TIMESTAMP
);
