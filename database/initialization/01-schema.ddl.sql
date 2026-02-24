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

CREATE SCHEMA auth;

CREATE TABLE auth.users (
    id uuid PRIMARY KEY,
    username text NOT NULL UNIQUE,
    password text NOT NULL
);

CREATE TABLE auth.api_keys (
    id uuid PRIMARY KEY,
    name text NOT NULL UNIQUE,
    owner text NOT NULL,
    value text NOT NULL UNIQUE
);