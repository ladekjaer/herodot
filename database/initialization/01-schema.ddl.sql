CREATE SCHEMA records;

CREATE TABLE records.bme280 (
    id uuid PRIMARY KEY,
    temperature real NOT NULL,
    pressure real NOT NULL,
    humidity real NOT NULL,
    timestamp timestamp with time zone NOT NULL
);

CREATE TABLE records.ds18b20 (
    id uuid PRIMARY KEY,
    device_name text NOT NULL,
    raw_reading integer NOT NULL,
    timestamp timestamp with time zone NOT NULL
);
