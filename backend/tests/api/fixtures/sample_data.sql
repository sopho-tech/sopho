CREATE TABLE customers (
    id integer PRIMARY KEY,
    name text NOT NULL,
    country text NOT NULL
);

CREATE TABLE orders (
    id integer PRIMARY KEY,
    customer_id integer NOT NULL REFERENCES customers (id),
    order_status text NOT NULL,
    amount numeric(10, 2) NOT NULL,
    ordered_at date NOT NULL
);

INSERT INTO customers (id, name, country) VALUES
    (1, 'Ada Lovelace', 'GB'),
    (2, 'Grace Hopper', 'US'),
    (3, 'Alan Turing', 'GB');

INSERT INTO orders (id, customer_id, order_status, amount, ordered_at) VALUES
    (1, 1, 'delivered', 120.50, '2026-01-05'),
    (2, 1, 'delivered', 80.00, '2026-02-11'),
    (3, 2, 'shipped', 300.25, '2026-02-20'),
    (4, 3, 'canceled', 45.75, '2026-03-02');
