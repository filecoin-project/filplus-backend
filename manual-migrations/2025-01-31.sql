CREATE TABLE public.comparable_applications
(
    client_address text NOT NULL,
    application jsonb NOT NULL,
    PRIMARY KEY (client_address)
);

ALTER TABLE IF EXISTS public.comparable_applications
    OWNER to postgres;