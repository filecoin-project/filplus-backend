CREATE TABLE comparable_applications
(
    client_address text NOT NULL,
    application jsonb NOT NULL,
    PRIMARY KEY (client_address)
);
