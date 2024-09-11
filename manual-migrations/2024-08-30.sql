CREATE TABLE autoallocations
(
    evm_wallet_address character varying(42) NOT NULL,
    last_allocation timestamp with time zone NOT NULL,
    PRIMARY KEY (evm_wallet_address)
);