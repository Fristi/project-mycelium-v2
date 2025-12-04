

create table stations (
    id UUID NOT NULL PRIMARY KEY,
    mac_addr BYTEA not null,
    name TEXT not null,
    description TEXT not null,
    location TEXT not null,
    watering_schedule JSON not null,
    user_id TEXT not null,
    created TIMESTAMPTZ not null,
    updated TIMESTAMPTZ
);

create table station_log (
    station_id UUID NOT NULL REFERENCES stations (id) ON DELETE CASCADE,
    occurred_on TIMESTAMPTZ NOT NULL,
    event JSON not null
);

create table station_measurements (
    station_id UUID NOT NULL REFERENCES stations (id) ON DELETE CASCADE,
    occurred_on TIMESTAMPTZ NOT NULL,
    battery_voltage decimal(5,2) NOT NULL,
    temperature decimal(5,2) NOT NULL,
    humidity decimal(5,2) NOT NULL,
    lux decimal(5,2) NOT NULL,
    soil_pf decimal(5,2) NOT NULL,
    tank_pf decimal(5,2) NOT NULL
);