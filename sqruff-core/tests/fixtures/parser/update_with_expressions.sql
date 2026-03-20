UPDATE inventory
SET quantity = quantity - 1, updated_at = '2024-06-15'
WHERE product_id = 42 AND quantity > 0;
