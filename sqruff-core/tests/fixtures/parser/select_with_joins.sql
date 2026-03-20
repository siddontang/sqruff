SELECT u.id, u.name, o.order_date, o.total, p.name AS product_name
FROM users u
INNER JOIN orders o ON u.id = o.user_id
LEFT JOIN order_items oi ON o.id = oi.order_id
LEFT JOIN products p ON oi.product_id = p.id
WHERE u.active = 1 AND o.total > 100
ORDER BY o.order_date DESC
LIMIT 50;
