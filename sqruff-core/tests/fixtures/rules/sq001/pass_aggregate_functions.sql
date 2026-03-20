SELECT COUNT(*), SUM(amount), AVG(price), MIN(created_at), MAX(updated_at)
FROM transactions
WHERE account_id = 42;
