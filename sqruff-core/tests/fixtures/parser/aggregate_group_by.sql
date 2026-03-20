SELECT department_id, COUNT(*) AS employee_count, AVG(salary) AS avg_salary
FROM employees
WHERE hire_date > '2020-01-01'
GROUP BY department_id
HAVING COUNT(*) > 5
ORDER BY avg_salary DESC;
