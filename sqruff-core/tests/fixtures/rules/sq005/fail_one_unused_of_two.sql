SELECT u.id, total FROM users u INNER JOIN orders o ON u.id = orders.user_id;
