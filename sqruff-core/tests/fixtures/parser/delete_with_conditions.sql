DELETE FROM audit_log WHERE created_at < '2023-01-01' AND level = 'debug';
