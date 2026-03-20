SELECT id, price * quantity + tax AS total, (discount / 100) * price AS savings
FROM line_items
WHERE price * quantity > 50;
